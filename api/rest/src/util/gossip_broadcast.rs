use anyhow::Result;
use hb_api_internal_gossip::InternalBroadcast;
use hb_dao::{change::ChangeDao, Db};

pub async fn save_change_data_and_broadcast(
    db: &Db,
    change_data: ChangeDao,
    internal_broadcast: &Option<InternalBroadcast>,
) -> Result<()> {
    change_data.db_upsert(db).await?;

    if let Some(internal_broadcast) = internal_broadcast {
        let internal_broadcast = internal_broadcast.clone();
        tokio::spawn((|| async move {
            if let Err(err) = internal_broadcast.broadcast(&change_data).await {
                hb_log::error(
                    None,
                    &format!(
                        "[ApiRestServer] Error when broadcasting change data (change_id: {}) to remote peer: {}", change_data.change_id(), err
                    ),
                );
            }
        })());
    }

    Ok(())
}
