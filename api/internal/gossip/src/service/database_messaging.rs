use std::sync::Arc;

use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};

use crate::message::database_action::DatabaseActionMessage;

pub struct DatabaseMessagingService {
    actions: Arc<Mutex<Vec<DatabaseActionMessage>>>,

    rx: mpsc::UnboundedReceiver<DatabaseActionMessage>,
}

impl DatabaseMessagingService {
    pub fn new() -> (Self, mpsc::UnboundedSender<DatabaseActionMessage>) {
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
        mut receiver: mpsc::UnboundedReceiver<DatabaseActionMessage>,
    ) {
        while let Some(message) = receiver.recv().await {
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
