use std::{fmt::Display, str::FromStr};

use backtrace::Backtrace;
use tracing::{debug, error, info, level_filters::LevelFilter, trace, warn};

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

pub fn trace<T: Display>(prefix: Option<&str>, msg: T) {
    match prefix {
        Some(prefix) => trace!("{prefix} {msg}"),
        None => trace!("🐾 {msg}"),
    }
}

pub fn debug<T: Display>(prefix: Option<&str>, msg: T) {
    match prefix {
        Some(prefix) => debug!("{prefix} {msg}"),
        None => debug!("🐞 {msg}"),
    };
}

pub fn info<T: Display>(prefix: Option<&str>, msg: T) {
    match prefix {
        Some(prefix) => info!("{prefix} {msg}"),
        None => info!("📢 {msg}"),
    };
}

pub fn warn<T: Display>(prefix: Option<&str>, msg: T) {
    match prefix {
        Some(prefix) => warn!("{prefix} {msg}"),
        None => warn!("⚠️ {msg}"),
    };
}

pub fn error<T: Display>(prefix: Option<&str>, msg: T) {
    let mut show_backtrace = false;
    if let Ok(var) = std::env::var("RUST_BACKTRACE") {
        if var == "1" {
            show_backtrace = true;
        }
    }
    match show_backtrace {
        true => match prefix {
            Some(prefix) => error!("{prefix} {msg}\n{:?}", Backtrace::new()),
            None => error!("🚨 {msg}\n{:?}", Backtrace::new()),
        },
        false => match prefix {
            Some(prefix) => error!("{prefix} {msg}"),
            None => error!("🚨 {msg}"),
        },
    };
}

pub fn panic<T: Display>(prefix: Option<&str>, msg: T) {
    match prefix {
        Some(prefix) => panic!("{prefix} {msg}"),
        None => panic!("☠️ {msg}"),
    };
}
