use std::{future::Future, net::SocketAddr, pin::Pin, time::Duration};

use anyhow::Result;
use futures::future::BoxFuture;
use tokio::{
    io::AsyncReadExt,
    net::TcpListener,
    sync::{mpsc, oneshot},
    time::timeout,
};

use crate::{handler::MessageHandler, message::Message};

pub struct GossipServer {
    builder: GossipServerBuilder,
}

impl GossipServer {
    pub fn new(address: SocketAddr, handler: MessageHandler) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        Self {
            builder: GossipServerBuilder::new(address, handler, cmd_tx, cmd_rx),
        }
    }

    pub fn run(self) -> GossipServerRunner {
        self.builder.run()
    }
}

struct GossipServerBuilder {
    address: SocketAddr,
    handler: MessageHandler,
    cmd_tx: mpsc::UnboundedSender<GossipServerCommand>,
    cmd_rx: mpsc::UnboundedReceiver<GossipServerCommand>,
}

impl GossipServerBuilder {
    fn new(
        address: SocketAddr,
        handler: MessageHandler,
        cmd_tx: mpsc::UnboundedSender<GossipServerCommand>,
        cmd_rx: mpsc::UnboundedReceiver<GossipServerCommand>,
    ) -> Self {
        Self {
            address,
            handler,
            cmd_tx,
            cmd_rx,
        }
    }

    fn run(self) -> GossipServerRunner {
        GossipServerRunner::new(self)
    }
}

pub struct GossipServerRunner {
    handle: GossipServerHandle,
    fut: BoxFuture<'static, Result<()>>,
}

impl GossipServerRunner {
    fn new(builder: GossipServerBuilder) -> Self {
        Self {
            handle: GossipServerHandle::new(builder.cmd_tx.clone()),
            fut: Box::pin(Self::run(builder)),
        }
    }

    pub fn handle(&self) -> GossipServerHandle {
        self.handle.clone()
    }

    async fn run(mut builder: GossipServerBuilder) -> Result<()> {
        let listener = TcpListener::bind(builder.address).await?;

        let mut stopping = (false, None);

        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    stopping.0 = true;
                }
                cmd = builder.cmd_rx.recv() => {
                    match cmd {
                        Some(cmd) => match cmd {
                            GossipServerCommand::Stop { completion } => {
                                stopping = (true, Some(completion));
                            }
                        },
                        None => {
                            stopping.0 = true;
                        }
                    }
                }
                result = listener.accept() => {
                    match result {
                        Ok(result) => {
                            let (mut tcp_stream, _) = result;
                            let message_handler = builder.handler.clone();
                            tokio::spawn((|| async move {
                                let mut buf = Vec::new();
                                let tcp_stream =
                                    match timeout(Duration::from_secs(5), tcp_stream.read_to_end(&mut buf)).await {
                                        Ok(tcp_stream) => tcp_stream,
                                        Err(err) => {
                                            hb_log::error(None, &format!("[ApiInternalGossip] {err}"));
                                            return;
                                        }
                                    };
                                match tcp_stream {
                                    Ok(read) => {
                                        if read > 0 {
                                            match Message::from_bytes(&buf) {
                                                Ok(message) => message.handle(message_handler),
                                                Err(err) => {
                                                    hb_log::error(None, &format!("[ApiInternalGossip] {err}"));
                                                    return;
                                                }
                                            };
                                        }
                                    }
                                    Err(err) => {
                                        hb_log::error(None, &format!("[ApiInternalGossip] {err}"));
                                        return;
                                    }
                                }
                            })());
                        }
                        Err(err) => {
                            hb_log::error(None, &format!("[ApiInternalGossip] {err}"));
                            stopping.0 = true;
                        }
                    }
                }
            }

            if stopping.0 {
                break;
            }
        }

        if let Some(completion) = stopping.1 {
            let _ = completion.send(());
        }

        Ok(())
    }
}

impl Future for GossipServerRunner {
    type Output = Result<()>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Pin::new(&mut Pin::into_inner(self).fut).poll(cx)
    }
}

#[derive(Clone)]
pub struct GossipServerHandle {
    cmd_tx: mpsc::UnboundedSender<GossipServerCommand>,
}

impl GossipServerHandle {
    fn new(cmd_tx: mpsc::UnboundedSender<GossipServerCommand>) -> Self {
        Self { cmd_tx }
    }

    pub async fn stop(&self) {
        let (tx, rx) = oneshot::channel();

        let _ = self
            .cmd_tx
            .send(GossipServerCommand::Stop { completion: tx });

        let _ = rx.await;
    }
}

enum GossipServerCommand {
    Stop { completion: oneshot::Sender<()> },
}
