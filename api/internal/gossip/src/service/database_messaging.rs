use std::{net::SocketAddr, sync::Arc};

use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};

use crate::message::{database_action::DatabaseActionMessage, MessageKind};

pub struct DatabaseMessagingService {
    actions: Arc<Mutex<Vec<DatabaseActionMessage>>>,

    rx: mpsc::UnboundedReceiver<(SocketAddr, MessageKind, DatabaseActionMessage)>,
}

impl DatabaseMessagingService {
    pub fn new() -> (
        Self,
        mpsc::UnboundedSender<(SocketAddr, MessageKind, DatabaseActionMessage)>,
    ) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self {
                actions: Arc::new(Mutex::new(Vec::new())),
                rx,
            },
            tx,
        )
    }

    pub fn run(self) -> JoinHandle<()> {
        hb_log::info(
            Some("🧩"),
            "[ApiInternalGossip] Running database messaging service",
        );

        tokio::spawn((|| async move {})())
    }

    async fn run_receiver_task(
        actions: Arc<Mutex<Vec<DatabaseActionMessage>>>,
        mut receiver: mpsc::UnboundedReceiver<(SocketAddr, MessageKind, DatabaseActionMessage)>,
    ) {
        while let Some((sender_address, kind, message)) = receiver.recv().await {
            let actions = actions.clone();
            tokio::spawn((|| async move {
                let actions = actions.lock().await;
            })());
        }
    }

    fn build_local_actions_buffer() -> Vec<DatabaseActionMessage> {
        let mut buffer = Vec::new();
        buffer
    }
}
