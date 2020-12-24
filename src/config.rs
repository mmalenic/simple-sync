use std::fs::{create_dir_all, read_to_string, write};
use std::io::{Error, ErrorKind};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::Path;

use directories::ProjectDirs;
use lazy_static::lazy_static;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use uuid::Uuid;

use crate::broadcast::{DEFAULT_MULTICAST_IPV4, DEFAULT_MULTICAST_IPV6, DEFAULT_PORT};
use crate::PROJECT_NAME;

const CONFIG_FILE: &str = "config.toml";

lazy_static! {
    static ref DEVICE_NAME: String = get_device_name();
    static ref DEVICE
}

#[derive(Debug, StructOpt)]
pub struct Options {
    #[structopt(long, short = "d", default_value = &get_device_id().to_string())]
    device_id: Uuid,

    #[structopt(long, short = "n", default_value = &DEVICE_NAME)]
    device_name: String,

    #[structopt(long, short = "p", default_value = &DEFAULT_PORT.to_string())]
    default_port: u16,

    #[structopt(long, short = "i", default_value = &DEFAULT_MULTICAST_IPV4.to_string())]
    default_multicast_ipv4: Ipv4Addr,

    #[structopt(long, short = "a", default_value = &DEFAULT_MULTICAST_IPV6.to_string())]
    default_multicast_ipv6: Ipv6Addr,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    device_id: Option<Uuid>
}

impl Config {
    fn deserialize(config: &String) -> Config {
        match toml::from_str::<Config>(&config) {
            Ok(config) => config,
            Err(e) => {
                warn!("Unable to deserialize config: {}", e);
                Config { device_id: None }
            }
        }
    }

    fn serialize(self) -> Option<String> {
        let serialized = match toml::to_string(&self) {
            Ok(id) => id,
            Err(e) => {
                warn!("Failed to serialize Config: {}", e);
                return None;
            }
        };
        Some(serialized)
    }
}

fn get_device_name() -> String {
    let hostname = hostname::get();
    if let Ok(x) = hostname {
        if let Ok(y) = x.into_string() {
            return y;
        }
    }
    Uuid::new_v4().to_string()
}

fn get_device_id() -> Uuid {
    let mut config = match read_config_file() {
        Ok(config) => Config::deserialize(&config),
        Err(e) => {
            warn!("Unable to read config file: {}", e);
            Config { device_id: None }
        }
    };
    match config.device_id {
        Some(id) => id,
        None => {
            let new_id = Uuid::new_v4();
            info!("Creating new device id: {}", new_id);
            config.device_id = Some(new_id);
            if let Some(serialized) = config.serialize() {
                write_config_file(serialized);
            }
            new_id
        }
    }
}

fn read_config_file() -> Result<String, Error> {
    let mut file_path = get_config_path()?.to_path_buf();
    file_path.push(CONFIG_FILE);
    read_to_string(file_path.as_path())
}

fn write_config_file(contents: String) -> Result<(), Error> {
    let mut file_path = get_config_path()?.to_path_buf();
    file_path.push(CONFIG_FILE);
    write(file_path.as_path(), contents)
}

fn get_config_path<'a>() -> Result<&'a Path, Error> {
    if let Some(project_dir) = ProjectDirs::from("", "", PROJECT_NAME) {
        if !project_dir.config_dir().exists() {
            create_dir_all(project_dir.config_dir())?
        }
        project_dir.config_dir();
    }
    Err(Error::new(ErrorKind::NotFound, "No valid home directory."))
}
