use std::sync::Arc;

use tokio::{sync::mpsc, task::JoinHandle};

use crate::{context::ApiMqttCtx, model::payload::Payload};

use self::record::record_service;

pub mod record;

pub struct Service {
    context: Arc<ApiMqttCtx>,
    rx: mpsc::UnboundedReceiver<Payload>,
}

impl Service {
    pub fn new(context: Arc<ApiMqttCtx>) -> (Self, mpsc::UnboundedSender<Payload>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (Self { context, rx }, tx)
    }

    pub fn run(mut self) -> JoinHandle<()> {
        tokio::spawn((|| async move {
            loop {
                if let Some(payload) = self.rx.recv().await {
                    record_service(&self.context, &payload).await;
                }
            }
        })())
    }
}
