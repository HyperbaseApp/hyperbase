use std::{fs, path::PathBuf, str::FromStr};

pub fn get() -> PathBuf {
    let config_path = match std::env::var("HB_CONFIG_PATH") {
        Ok(path) => path,
        Err(_) => "config.yml".to_owned(),
    };

    if let Err(_) = fs::metadata(&config_path) {
        panic!("config.yml file specified in HB_CONFIG_PATH environment variable or current directory must exist")
    }

    PathBuf::from_str(&config_path).unwrap()
}
