use std::{net::SocketAddr, sync::Arc, time::Duration};

use hb_dao::{
    change::{ChangeDao, ChangeState, ChangeTable},
    local_info::LocalInfoDao,
    remote_sync::RemoteSyncDao,
    Db,
};
use rand::{prelude::SliceRandom, Rng};
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use uuid::Uuid;

use crate::{
    client,
    config::database_messaging::DatabaseMessagingConfig,
    message::{
        content::{ContentChangeModel, ContentMessage},
        header::HeaderMessage,
        Message,
    },
    view::View,
};

pub struct DatabaseMessagingService {
    config: DatabaseMessagingConfig,
    db: Arc<Db>,
    view: Arc<Mutex<View>>,

    header_rx: mpsc::UnboundedReceiver<(SocketAddr, Uuid, Uuid, HeaderMessage)>,
    content_rx: mpsc::UnboundedReceiver<(SocketAddr, Uuid, Uuid, ContentMessage)>,
}

impl DatabaseMessagingService {
    pub fn new(
        config: DatabaseMessagingConfig,
        db: Arc<Db>,
        view: Arc<Mutex<View>>,
    ) -> (
        Self,
        mpsc::UnboundedSender<(SocketAddr, Uuid, Uuid, HeaderMessage)>,
        mpsc::UnboundedSender<(SocketAddr, Uuid, Uuid, ContentMessage)>,
    ) {
        let (header_tx, header_rx) = mpsc::unbounded_channel();
        let (content_tx, content_rx) = mpsc::unbounded_channel();
        (
            Self {
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
            let local_id = match LocalInfoDao::db_select(&self.db).await {
                Ok(data) => *data.id(),
                Err(_) => {
                    let local_info_data = LocalInfoDao::new();
                    if let Err(err) = local_info_data.db_insert(&self.db).await {
                        hb_log::error(
                            None,
                            format!(
                                "[ApiInternalGossip] Error select or insert local_info data: {err}"
                            ),
                        );
                        return;
                    }
                    *local_info_data.id()
                }
            };

            tokio::join!(
                Self::run_receiver_task(
                    local_id,
                    self.config,
                    self.db.clone(),
                    self.header_rx,
                    self.content_rx
                ),
                Self::run_sender_task(local_id, self.config, self.db, self.view)
            );
        })())
    }

    async fn run_receiver_task(
        local_id: Uuid,
        config: DatabaseMessagingConfig,
        db: Arc<Db>,
        mut header_receiver: mpsc::UnboundedReceiver<(SocketAddr, Uuid, Uuid, HeaderMessage)>,
        mut content_receiver: mpsc::UnboundedReceiver<(SocketAddr, Uuid, Uuid, ContentMessage)>,
    ) {
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
                                    let changes_data = match ChangeDao::db_select_many_from_updated_at_and_after_change_id_with_limit_asc(&db, &from_time, &last_change_id, config.actions_size()).await {
                                        Ok(data) => data,
                                        Err(err) => {
                                            hb_log::error(
                                                    None,
                                                    format!(
                                                        "[ApiInternalGossip] Error select many changes data: {err}"
                                                    ),
                                                );
                                            return;
                                        }
                                    };
                                    let mut content_changes_data = Vec::with_capacity(changes_data.len());
                                    for change_data in &changes_data {
                                        content_changes_data.push(*change_data.id());
                                    }
                                    match client::send(
                                        &sender_address,
                                        Message::Header {
                                            from: local_id,
                                            to: from,
                                            value: HeaderMessage::Response {
                                                change_ids: content_changes_data,
                                            },
                                        },
                                    )
                                    .await
                                    {
                                        Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Header message response sent successfully to {sender_address} ({written} bytes)")),
                                        Err(err) => hb_log::warn(None, &format!("[ApiInternalGossip] Header message response failed to send to {sender_address} due to error: {err}")),
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
                                            hb_log::error(
                                                        None,
                                                        format!(
                                                            "[ApiInternalGossip] Error select many changes data: {err}"
                                                        ),
                                                    );
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
                                    match client::send(&sender_address, Message::Content { from: local_id, to: from, value: ContentMessage::Request { change_ids: missing_change_ids } }).await{
                                        Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Content message request sent successfully to {sender_address} ({written} bytes)")),
                                        Err(err) => hb_log::warn(None, &format!("[ApiInternalGossip] Content message request failed to send to {sender_address} due to error: {err}")),
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
                                            hb_log::error(
                                                    None,
                                                    format!(
                                                        "[ApiInternalGossip] Error select many changes data: {err}"
                                                    ),
                                                );
                                            return;
                                        }
                                    };
                                    let mut content_changes_data = Vec::with_capacity(changes_data.len());
                                    for change_data in &changes_data {
                                        content_changes_data.push(ContentChangeModel::new(
                                            &change_data.table().to_string(),
                                            change_data.id(),
                                            change_data.state().to_str(),
                                            change_data.updated_at(),
                                            change_data.id(),
                                        ))
                                    }
                                    match client::send(
                                        &sender_address,
                                        Message::Content {
                                            from: local_id,
                                            to: from,
                                            value: ContentMessage::Response {
                                                changes_data: content_changes_data,
                                            },
                                        },
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
                                        let change_data_table = match ChangeTable::from_str(data.table()) {
                                            Ok(data) => data,
                                            Err(err) => {
                                                hb_log::error(
                                                    None,
                                                    format!(
                                                        "[ApiInternalGossip] Error converting string to table name: {err}"
                                                    ),
                                                );
                                                return;
                                            }
                                        };
                                        let change_data_state = match ChangeState::from_str(data.state()) {
                                            Ok(data) => data,
                                            Err(err) => {
                                                hb_log::error(
                                                    None,
                                                    format!(
                                                        "[ApiInternalGossip] Error converting string to change state: {err}"
                                                    ),
                                                );
                                                return;
                                            }
                                        };
                                        let change_data = ChangeDao::raw_new(
                                            &change_data_table,
                                            data.id(),
                                            &change_data_state,
                                            data.updated_at(),
                                            data.change_id(),
                                        );
                                        if let Err(err) = change_data.db_upsert(&db).await {
                                            hb_log::warn(
                                                None,
                                                format!(
                                                    "[ApiInternalGossip] Error upsert change data: {err}"
                                                ),
                                            );
                                            return;
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

    async fn run_sender_task(
        local_id: Uuid,
        config: DatabaseMessagingConfig,
        db: Arc<Db>,
        view: Arc<Mutex<View>>,
    ) {
        loop {
            let view = view.lock().await;
            if let Some(peer) = view.select_peer() {
                let remotes_sync_data = match RemoteSyncDao::db_select_many_by_address(
                    &db,
                    peer.address(),
                )
                .await
                {
                    Ok(data) => {
                        if !data.is_empty() {
                            data
                        } else {
                            hb_log::error(
                            None,
                            format!(
                                "[ApiInternalGossip] Error select many remotes data by address: remotes is empty"
                            ),
                        );
                            return;
                        }
                    }
                    Err(err) => {
                        hb_log::error(
                                None,
                                format!(
                                    "[ApiInternalGossip] Error select many remotes data by address: {err}"
                                ),
                            );
                        return;
                    }
                };
                let remote_sync_data = remotes_sync_data.choose(&mut rand::thread_rng()).unwrap();
                match client::send(
                    peer.address(),
                    Message::Header {
                        from: local_id,
                        to: *remote_sync_data.remote_id(),
                        value: HeaderMessage::Request {
                            from_time: *remote_sync_data.last_data_sync(),
                            last_change_id: *remote_sync_data.last_change_id(),
                        },
                    },
                )
                .await
                {
                    Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Header message request sent successfully to {} ({} bytes)", peer.address(),written)),
                    Err(err) => hb_log::warn(None, &format!("[ApiInternalGossip] Header message request failed to send to {} due to error: {}", peer.address(), err)),
             }
            }

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
}
