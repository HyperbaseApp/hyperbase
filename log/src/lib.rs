use std::{fmt::Debug, str::FromStr};

use tracing::{debug, error, info, level_filters::LevelFilter, warn};

pub fn init(display_level: &bool, level_filter: &str) {
    let level_filter = match LevelFilter::from_str(level_filter) {
        Ok(level) => level,
        Err(err) => panic!("{err}"),
    };

    tracing_subscriber::fmt()
        .with_level(*display_level)
        .with_max_level(level_filter)
        .init();
}

pub fn debug<T: Debug>(prefix: Option<&str>, msg: &T) {
    match prefix {
        Some(prefix) => debug!("{prefix} {msg:#?}"),
        None => debug!("🐞 {msg:#?}"),
    };
}

pub fn error<T: Debug>(prefix: Option<&str>, msg: &T) {
    match prefix {
        Some(prefix) => error!("{prefix} {msg:#?}"),
        None => error!("🚨 {msg:#?}"),
    };
}

pub fn info<T: Debug>(prefix: Option<&str>, msg: &T) {
    match prefix {
        Some(prefix) => info!("{prefix} {msg:#?}"),
        None => info!("📢 {msg:#?}"),
    };
}

pub fn warn<T: Debug>(prefix: Option<&str>, msg: &T) {
    match prefix {
        Some(prefix) => warn!("{prefix} {msg:#?}"),
        None => warn!("⚠️ {msg:#?}"),
    };
}
