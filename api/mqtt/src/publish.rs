use std::sync::Arc;

use ntex::{fn_service, ServiceFactory};
use ntex_mqtt::{v3, v5};

use crate::{
    context::ApiMqttCtx,
    error_handler::ServerError,
    service::{v3::pub_record::v3_pub_record_api, v5::pub_record::v5_pub_record_api},
    session::Session,
};

pub fn v3_publish(ctx: &Arc<ApiMqttCtx>) -> v3::Router<Session, ServerError> {
    let mut router = v3::Router::new(
        fn_service(|_: v3::Publish| async { Ok(()) }).map_init_err(|_| ServerError),
    );

    router = v3_pub_record_api(ctx.clone(), router);

    router
}

pub fn v5_publish(ctx: &Arc<ApiMqttCtx>) -> v5::Router<Session, ServerError> {
    let mut router = v5::Router::new(
        fn_service(|publish: v5::Publish| async { Ok(publish.ack()) })
            .map_init_err(|_| ServerError),
    );

    router = v5_pub_record_api(ctx.clone(), router);

    router
}
