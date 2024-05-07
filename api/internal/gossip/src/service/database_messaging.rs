use std::{net::SocketAddr, sync::Arc, time::Duration};

use chrono::Utc;
use hb_dao::{change::ChangeDao, remote_sync::RemoteSyncDao, Db};
use rand::Rng;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use uuid::Uuid;

use crate::{
    client,
    config::database_messaging::DatabaseMessagingConfig,
    message::{
        content::{
            ContentChangeModel, ContentChannelReceiver, ContentChannelSender, ContentMessage,
        },
        header::{HeaderChannelReceiver, HeaderChannelSender, HeaderMessage},
        Message, MessageV,
    },
    view::View,
};

pub struct DatabaseMessagingService {
    local_id: Uuid,
    local_address: SocketAddr,
    config: DatabaseMessagingConfig,
    db: Arc<Db>,
    view: Arc<Mutex<View>>,

    header_rx: HeaderChannelReceiver,
    content_rx: ContentChannelReceiver,
}

impl DatabaseMessagingService {
    pub fn new(
        local_id: Uuid,
        local_address: SocketAddr,
        config: DatabaseMessagingConfig,
        db: Arc<Db>,
        view: Arc<Mutex<View>>,
    ) -> (Self, HeaderChannelSender, ContentChannelSender) {
        let (header_tx, header_rx) = mpsc::unbounded_channel();
        let (content_tx, content_rx) = mpsc::unbounded_channel();
        (
            Self {
                local_id,
                local_address,
                config,
                db,
                header_rx,
                content_rx,
                view,
            },
            header_tx,
            content_tx,
        )
    }

    pub fn run(self) -> JoinHandle<()> {
        hb_log::info(
            Some("🧩"),
            "[ApiInternalGossip] Running database messaging service",
        );

        tokio::spawn((|| async move {
            tokio::join!(
                Self::run_receiver_task(
                    self.local_id,
                    self.local_address,
                    self.config,
                    self.db.clone(),
                    self.view.clone(),
                    self.header_rx,
                    self.content_rx
                ),
                Self::run_sender_task(
                    self.local_id,
                    self.local_address,
                    self.config,
                    self.db,
                    self.view
                )
            );
        })())
    }

    async fn run_receiver_task(
        local_id: Uuid,
        local_address: SocketAddr,
        config: DatabaseMessagingConfig,
        db: Arc<Db>,
        view: Arc<Mutex<View>>,
        mut header_receiver: HeaderChannelReceiver,
        mut content_receiver: ContentChannelReceiver,
    ) {
        loop {
            tokio::select! {
                msg = header_receiver.recv() => {
                    if let Some((sender_address, from, to, header)) = msg {
                        if to == local_id {
                            let db = db.clone();
                            tokio::spawn((|| async move {
                                match header {
                                    HeaderMessage::Request {
                                        from_time,
                                        last_change_id,
                                    } => {
                                        let time_threshold = Utc::now() - Duration::from_secs(1);
                                        let changes_data = match ChangeDao::db_select_many_from_updated_at_and_after_change_id_with_limit_asc(&db, &from_time, &last_change_id, config.actions_size()).await {
                                            Ok(data) => data,
                                            Err(err) => {
                                                hb_log::error(None, format!("[ApiInternalGossip] Error select many changes data: {err}"));
                                                return;
                                            }
                                        };
                                        let mut content_changes_data = Vec::with_capacity(changes_data.len());
                                        for change_data in &changes_data {
                                            if *change_data.updated_at() < time_threshold {
                                                content_changes_data.push(*change_data.id());
                                            }
                                        }
                                        if !content_changes_data.is_empty() {
                                            match client::send(
                                                &sender_address,
                                                Message::new(
                                                    local_address,
                                                    MessageV::Header {
                                                        from: local_id,
                                                        to: from,
                                                        data: HeaderMessage::Response {
                                                            change_ids: content_changes_data,
                                                        },
                                                    }
                                                ),
                                            )
                                            .await
                                            {
                                                Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Header message response sent successfully to {sender_address} ({written} bytes)")),
                                                Err(err) => hb_log::warn(None, &format!("[ApiInternalGossip] Header message response failed to send to {sender_address} due to error: {err}")),
                                            }
                                        }
                                    }
                                    HeaderMessage::Response { change_ids } => {
                                        let changes_data = match ChangeDao::db_select_many_by_change_ids_asc(
                                            &db,
                                            &change_ids,
                                        )
                                        .await
                                        {
                                            Ok(data) => data,
                                            Err(err) => {
                                                hb_log::error(None, format!("[ApiInternalGossip] Error select many changes data: {err}"));
                                                return;
                                            }
                                        };
                                        let mut missing_change_ids = Vec::with_capacity(change_ids.len());
                                        for change_id in &change_ids {
                                            let mut exists = false;
                                            for change_data in &changes_data {
                                                if change_data.change_id() == change_id {
                                                    exists = true;
                                                    break;
                                                }
                                            }
                                            if !exists {
                                                missing_change_ids.push(*change_id);
                                            }
                                        }
                                        if !missing_change_ids.is_empty() {
                                            match client::send(&sender_address, Message::new(local_address, MessageV::Content { from: local_id, to: from, data: ContentMessage::Request { change_ids: missing_change_ids } })).await{
                                                Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Content message request sent successfully to {sender_address} ({written} bytes)")),
                                                Err(err) => hb_log::warn(None, &format!("[ApiInternalGossip] Content message request failed to send to {sender_address} due to error: {err}")),
                                            }
                                        }
                                    }
                                }
                            })());
                        }
                    }
                }
                msg = content_receiver.recv() => {
                    if let Some((sender_address, from, to, message)) = msg {
                        if to == local_id {
                            let db = db.clone();
                            let view = view.lock().await;
                            let view = view.clone();
                            tokio::spawn((|| async move {
                                match message {
                                    ContentMessage::Request { change_ids } => {
                                        let changes_data = match ChangeDao::db_select_many_by_change_ids_asc(
                                            &db,
                                            &change_ids,
                                        )
                                        .await
                                        {
                                            Ok(data) => data,
                                            Err(err) => {
                                                hb_log::error(None, format!("[ApiInternalGossip] Error select many changes data: {err}"));
                                                return;
                                            }
                                        };
                                        let mut content_changes_data = Vec::with_capacity(changes_data.len());
                                        for change_data in &changes_data {
                                            let content_change_data = match ContentChangeModel::from_change_dao(
                                                &db,
                                                change_data,
                                            )
                                            .await
                                            {
                                                Ok(data) => data,
                                                Err(err) => {
                                                    hb_log::error(None, format!("[ApiInternalGossip] Error convert change dao to content change data: {err}"));
                                                    return;
                                                }
                                            };
                                            content_changes_data.push(content_change_data)
                                        }
                                        match client::send(
                                            &sender_address,
                                            Message::new(
                                                local_address,
                                                MessageV::Content {
                                                    from: local_id,
                                                    to: from,
                                                    data: ContentMessage::Response {
                                                        changes_data: content_changes_data,
                                                    },
                                                }
                                            ),
                                        )
                                        .await
                                        {
                                            Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Content message response sent successfully to {sender_address} ({written} bytes)")),
                                            Err(err) => hb_log::warn(None, &format!("[ApiInternalGossip] Content message response failed to send to {sender_address} due to error: {err}")),
                                        }
                                    }
                                    ContentMessage::Response { mut changes_data } => {
                                        changes_data.sort_by(|a, b| {
                                            if a.updated_at() == b.updated_at() {
                                                a.change_id().cmp(b.change_id())
                                            } else {
                                                a.updated_at().cmp(b.updated_at())
                                            }
                                        });
                                        for data in &changes_data {
                                            if let Err(err) = data.handle(&db).await {
                                                hb_log::warn(None, format!("[ApiInternalGossip] Error handle content change data: {err}"));
                                                return;
                                            }
                                        }
                                        if let Some(last_change_data) = changes_data.last() {
                                            let remote_sync_data = RemoteSyncDao::new(
                                                &from,
                                                &sender_address,
                                                last_change_data.updated_at(),
                                                last_change_data.change_id(),
                                            );
                                            if let Err(err) = remote_sync_data.db_upsert(&db).await {
                                                hb_log::error(None, format!("[ApiInternalGossip] Error upsert remote_sync data: {err}"));
                                                return;
                                            }
                                            Self::send_request_header(&local_address, &local_id, &db, view.clone()).await;
                                        }
                                    }
                                    ContentMessage::Broadcast { change_data } => {
                                        if let Err(err) = change_data.handle(&db).await {
                                            hb_log::warn(None, format!("[ApiInternalGossip] Error handle content change data: {err}"));
                                            return;
                                        }
                                        let mut selected_remotes = Vec::new();
                                        let mut selecting_count = 0;
                                        loop {
                                            if selected_remotes.len() < (*config.max_broadcast() as usize)
                                                && (selecting_count as usize) < view.len_peers()
                                            {
                                                selecting_count += 1;
                                            } else {
                                                break;
                                            }
                                            if let Some(remote_sync_data) = view.select_remote_sync(&db).await {
                                                match remote_sync_data {
                                                    Ok(remote_sync_data) => {
                                                        if selected_remotes.contains(remote_sync_data.remote_id()) {
                                                            continue;
                                                        }
                                                        selected_remotes.push(*remote_sync_data.remote_id());
                                                        match client::send(
                                                            remote_sync_data.remote_address(),
                                                            Message::new(
                                                                local_address,
                                                                MessageV::Content {
                                                                    from: local_id,
                                                                    to: *remote_sync_data.remote_id(),
                                                                    data: ContentMessage::Broadcast { change_data: change_data.clone() },
                                                                }
                                                            )
                                                        )
                                                        .await
                                                        {
                                                            Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Broadcast message sent successfully to {} ({} bytes)", remote_sync_data.remote_address(), written)),
                                                            Err(err) => hb_log::warn(None, &format!("[ApiInternalGossip] Broadcast message failed to send to {} due to error: {}", remote_sync_data.remote_address(), err)),
                                                        }
                                                    }
                                                    Err(err) => {
                                                        hb_log::warn(None, &format!("[ApiInternalGossip] Failed to get remote sync: {err}"));
                                                        return;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            })());
                        }
                    }
                }

            }
        }
    }

    async fn run_sender_task(
        local_id: Uuid,
        local_address: SocketAddr,
        config: DatabaseMessagingConfig,
        db: Arc<Db>,
        view: Arc<Mutex<View>>,
    ) {
        loop {
            let view = view.lock().await;
            Self::send_request_header(&local_address, &local_id, &db, view.clone()).await;

            let sleep_duration_deviation = match config.period_deviation() {
                0 => 0,
                val => rand::thread_rng().gen_range(0..=*val),
            };
            let sleep_duration = config.period() + sleep_duration_deviation;

            hb_log::info(
                None,
                format!(
                    "[ApiInternalGossip] Next header message request is after {sleep_duration} ms"
                ),
            );

            tokio::time::sleep(Duration::from_millis(sleep_duration)).await;
        }
    }

    async fn send_request_header(local_address: &SocketAddr, local_id: &Uuid, db: &Db, view: View) {
        if let Some(remote_sync_data) = view.select_remote_sync(db).await {
            match remote_sync_data {
                Ok(remote_sync_data) => {
                    match client::send(
                            remote_sync_data.remote_address(),
                            Message::new(
                                *local_address,
                                MessageV::Header {
                                    from: *local_id,
                                    to: *remote_sync_data.remote_id(),
                                    data: HeaderMessage::Request {
                                        from_time: *remote_sync_data.last_data_sync(),
                                        last_change_id: *remote_sync_data.last_change_id(),
                                    },
                                }
                            )
                        )
                        .await
                        {
                            Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Header message request sent successfully to {} ({} bytes)", remote_sync_data.remote_address(), written)),
                            Err(err) => hb_log::warn(None, &format!("[ApiInternalGossip] Header message request failed to send to {} due to error: {}", remote_sync_data.remote_address(), err)),
                        }
                }
                Err(err) => {
                    hb_log::warn(
                        None,
                        &format!("[ApiInternalGossip] Failed to get remote sync: {err}"),
                    );
                }
            }
        } else {
            hb_log::warn(
                None,
                "[ApiInternalGossip] No remote found for header message request",
            );
        }
    }
}
