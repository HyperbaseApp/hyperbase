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

    status: Arc<Mutex<DatabaseMessagingStatus>>,

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
                view,
                status: Arc::new(Mutex::new(DatabaseMessagingStatus::default())),
                header_rx,
                content_rx,
            },
            header_tx,
            content_tx,
        )
    }

    pub async fn run(self) {
        hb_log::info(
            Some("🧩"),
            "[ApiInternalGossip] Running database messaging service",
        );

        tokio::try_join!(
            Self::run_receiver_task(
                self.local_id,
                self.local_address,
                self.config,
                self.db.clone(),
                self.view.clone(),
                self.status.clone(),
                self.header_rx,
                self.content_rx
            ),
            Self::run_sender_task(
                self.local_id,
                self.local_address,
                self.config,
                self.db,
                self.view,
                self.status
            )
        )
        .unwrap();
    }

    fn run_receiver_task(
        local_id: Uuid,
        local_address: SocketAddr,
        config: DatabaseMessagingConfig,
        db: Arc<Db>,
        view: Arc<Mutex<View>>,
        status: Arc<Mutex<DatabaseMessagingStatus>>,
        mut header_receiver: HeaderChannelReceiver,
        mut content_receiver: ContentChannelReceiver,
    ) -> JoinHandle<()> {
        tokio::spawn((|| async move {
            loop {
                tokio::select! {
                    msg = header_receiver.recv() => {
                        if let Some((sender_address, from, to, header)) = msg {
                            if to == local_id {
                                match header {
                                    HeaderMessage::Request {
                                        from_time,
                                        last_change_id,
                                    } => {
                                        hb_log::info(None, &format!("[ApiInternalGossip] Header message request received: sender: {sender_address}, from_id: {from}, to_id: {to}, local_id: {local_id}, from_time: {from_time}, last_change_id: {last_change_id}"));
                                        let time_threshold = Utc::now() - Duration::from_secs(1);
                                        let changes_data = match ChangeDao::db_select_many_from_timestamp_and_after_change_id_with_limit_asc(&db, &from_time, &last_change_id, config.actions_size()).await {
                                            Ok(data) => data,
                                            Err(err) => {
                                                hb_log::error(None, format!("[ApiInternalGossip] Error select many changes data: {err}"));
                                                continue;
                                            }
                                        };
                                        let mut content_changes_data = Vec::with_capacity(changes_data.len());
                                        for change_data in &changes_data {
                                            if *change_data.timestamp() < time_threshold {
                                                content_changes_data.push(*change_data.change_id());
                                            }
                                        }
                                        if !content_changes_data.is_empty() {
                                            let content_changes_len = content_changes_data.len();
                                            let first_content_change_id = content_changes_data.first().unwrap().clone();
                                            let last_content_change_id = content_changes_data.last().unwrap().clone();
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
                                                Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Header message response sent successfully to {} ({} data (id from {} to {}), {} bytes)", sender_address, content_changes_len, first_content_change_id, last_content_change_id, written)),
                                                Err(err) => hb_log::warn(None, &format!("[ApiInternalGossip] Header message response failed to send to {sender_address} due to error: {err}")),
                                            }
                                        }
                                    }
                                    HeaderMessage::Response { change_ids } => {
                                        hb_log::info(None, &format!("[ApiInternalGossip] Header message response received: sender: {}, from_id: {}, to_id: {}, local_id: {}, last_change_id: {} data", sender_address, from, to, local_id, change_ids.len()));
                                        let changes_data = match ChangeDao::db_select_many_by_change_ids_asc(
                                            &db,
                                            &change_ids,
                                        )
                                        .await
                                        {
                                            Ok(data) => data,
                                            Err(err) => {
                                                hb_log::error(None, format!("[ApiInternalGossip] Error select many changes data: {err}"));
                                                continue;
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
                                            let missing_change_ids_len = missing_change_ids.len();
                                            match client::send(&sender_address, Message::new(local_address, MessageV::Content { from: local_id, to: from, data: ContentMessage::Request { change_ids: missing_change_ids } })).await{
                                                Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Content message request sent successfully to {sender_address} ({missing_change_ids_len} data, {written} bytes)")),
                                                Err(err) => hb_log::warn(None, &format!("[ApiInternalGossip] Content message request failed to send to {sender_address} due to error: {err}")),
                                            }
                                        } else if let Some(last_change_data) = changes_data.last() {
                                            let remote_sync_data = RemoteSyncDao::new(
                                                &from,
                                                &sender_address,
                                                last_change_data.timestamp(),
                                                last_change_data.change_id(),
                                            );
                                            match remote_sync_data.db_upsert(&db).await {
                                                Ok(_) => hb_log::info(None, &format!("[ApiInternalGossip] Upsert remote_sync: remote_id: {}, address: {}, last_data_sync: {}, last_change_id: {}", from, sender_address, last_change_data.timestamp(), last_change_data.change_id())),
                                                Err(err)=> hb_log::error(None, &format!("[ApiInternalGossip] Failed to upsert remote_sync data: {err}")),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    msg = content_receiver.recv() => {
                        if let Some((sender_address, from, to, message)) = msg {
                            if to == local_id {
                                let db = db.clone();
                                let view_mutex = view.lock().await;
                                let view = view_mutex.clone();
                                let status = status.clone();
                                drop(view_mutex);
                                match message {
                                    ContentMessage::Request { change_ids } => {
                                        hb_log::info(None, &format!("[ApiInternalGossip] Content message request received: sender: {}, from_id: {}, to_id: {}, local_id: {}, change_ids: {} data", sender_address, from, to, local_id, change_ids.len()));
                                        let changes_data = match ChangeDao::db_select_many_by_change_ids_asc(
                                            &db,
                                            &change_ids,
                                        )
                                        .await
                                        {
                                            Ok(data) => data,
                                            Err(err) => {
                                                hb_log::error(None, format!("[ApiInternalGossip] Error select many changes data: {err}"));
                                                continue;
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
                                                    continue;
                                                }
                                            };
                                            content_changes_data.push(content_change_data)
                                        }
                                        let content_changes_len = content_changes_data.len();
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
                                            Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Content message response sent successfully to {sender_address} ({content_changes_len} data, {written} bytes)")),
                                            Err(err) => hb_log::warn(None, &format!("[ApiInternalGossip] Content message response failed to send to {sender_address} due to error: {err}")),
                                        }
                                    }
                                    ContentMessage::Response { mut changes_data } => {
                                        hb_log::info(None, &format!("[ApiInternalGossip] Content message response received: sender: {}, from_id: {}, to_id: {}, local_id: {}, changes_data: {} data", sender_address, from, to, local_id, changes_data.len()));
                                        let mut status_mtx = status.lock().await;
                                        status_mtx.set_to_syncing();
                                        drop(status_mtx);

                                        changes_data.sort_by(|a, b| {
                                            if a.timestamp() == b.timestamp() {
                                                a.change_id().cmp(b.change_id())
                                            } else {
                                                a.timestamp().cmp(b.timestamp())
                                            }
                                        });
                                        let mut last_change_data = None;
                                        for data in &changes_data {
                                            last_change_data = Some(data);
                                            if let Err(err) = data.handle(&db).await {
                                                hb_log::warn(None, format!("[ApiInternalGossip] Error handle content change data: {err}"));
                                                break;
                                            }
                                        }
                                        if let Some(last_change_data) = last_change_data {
                                            let remote_sync_data = RemoteSyncDao::new(
                                                &from,
                                                &sender_address,
                                                last_change_data.timestamp(),
                                                last_change_data.change_id(),
                                            );
                                            if let Err(err) = remote_sync_data.db_upsert(&db).await {
                                                hb_log::error(None, format!("[ApiInternalGossip] Error upsert remote_sync data: {err}"));
                                                continue;
                                            }
                                            Self::send_request_header(&local_address, &local_id, &remote_sync_data).await;
                                        } else {
                                            let mut status_mtx = status.lock().await;
                                            status_mtx.set_to_not_syncing();
                                        }
                                    }
                                    ContentMessage::Broadcast { change_data } => {
                                        hb_log::info(None, &format!("[ApiInternalGossip] Content message broadcast received: sender: {}, from_id: {}, to_id: {}, local_id: {}, change_data: {} data", sender_address, from, to, local_id, change_data.change_id()));
                                        if let Err(err) = change_data.handle(&db).await {
                                            hb_log::warn(None, format!("[ApiInternalGossip] Error handle content change data: {err}"));
                                            continue;
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
                                                            Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Broadcast message sent successfully to {} (change_id {}, {} bytes)", remote_sync_data.remote_address(), change_data.change_id(), written)),
                                                            Err(err) => hb_log::warn(None, &format!("[ApiInternalGossip] Broadcast message failed to send to {} due to error: {}", remote_sync_data.remote_address(), err)),
                                                        }
                                                    }
                                                    Err(err) => {
                                                        hb_log::warn(None, &format!("[ApiInternalGossip] Failed to get remote sync: {err}"));
                                                        continue;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                }
            }
        })())
    }

    fn run_sender_task(
        local_id: Uuid,
        local_address: SocketAddr,
        config: DatabaseMessagingConfig,
        db: Arc<Db>,
        view: Arc<Mutex<View>>,
        status: Arc<Mutex<DatabaseMessagingStatus>>,
    ) -> JoinHandle<()> {
        tokio::spawn((|| async move {
            loop {
                let mut status = status.lock().await;
                if status.allow_send_request_header_attemp() {
                    let view = view.lock().await;
                    if let Some(remote_sync_data) = view.select_remote_sync(&db).await {
                        match remote_sync_data {
                            Ok(remote_sync_data) => {
                                Self::send_request_header(
                                    &local_address,
                                    &local_id,
                                    &remote_sync_data,
                                )
                                .await;
                            }
                            Err(err) => {
                                hb_log::warn(
                                    None,
                                    &format!(
                                        "[ApiInternalGossip] Failed to get remote sync: {err}"
                                    ),
                                );
                            }
                        }
                    } else {
                        hb_log::warn(
                            None,
                            "[ApiInternalGossip] No remote found for header message request",
                        );
                    }
                    drop(view);
                }
                drop(status);

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
        })())
    }

    async fn send_request_header(
        local_address: &SocketAddr,
        local_id: &Uuid,
        remote_sync_data: &RemoteSyncDao,
    ) {
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
            Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Header message request sent successfully to {} (from {}, last_id {}, {} bytes)", remote_sync_data.remote_address(), remote_sync_data.last_data_sync(), remote_sync_data.last_change_id(), written)),
            Err(err) => hb_log::warn(None, &format!("[ApiInternalGossip] Header message request failed to send to {} due to error: {}", remote_sync_data.remote_address(), err)),
        }
    }
}

struct DatabaseMessagingStatus {
    is_syncing: bool,
    send_request_header_attempt_count_when_syncing: u8,
}

impl DatabaseMessagingStatus {
    fn allow_send_request_header_attemp(&mut self) -> bool {
        if self.is_syncing {
            if self.send_request_header_attempt_count_when_syncing < 3 {
                self.send_request_header_attempt_count_when_syncing += 1;
                false
            } else {
                self.reset();
                true
            }
        } else {
            true
        }
    }

    fn set_to_syncing(&mut self) {
        if !self.is_syncing {
            self.is_syncing = true;
        }
    }

    fn set_to_not_syncing(&mut self) {
        if self.is_syncing {
            self.reset();
        }
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}

impl Default for DatabaseMessagingStatus {
    fn default() -> Self {
        Self {
            is_syncing: false,
            send_request_header_attempt_count_when_syncing: 0,
        }
    }
}
