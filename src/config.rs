use std::ffi::OsString;
use std::fs::{create_dir_all, read_to_string, write};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;
use std::str::FromStr;

use clap::{App, Arg, ArgMatches};
use directories::ProjectDirs;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::PROJECT_NAME;

const CONFIG_FILE: &'static str = "config.toml";

// Names for command line options
const OPTION_CONFIG_FILE: &'static str = "config-file";
const OPTION_DEVICE_ID: &'static str = "device-id";
const OPTION_DEVICE_NAME: &'static str = "device-name";
const OPTION_USE_HOSTNAME: &'static str = "use-hostname";
const OPTION_PORT: &'static str = "port";
const OPTION_MULTICAST_IPV4: &'static str = "multicast-ipv4";
const OPTION_MULTICAST_IPV6: &'static str = "multicast-ipv6";

#[derive(Debug)]
pub struct Options {
    device_id: Uuid,
    device_name: OsString,
    use_hostname: bool,
    port: u16,
    multicast_ipv4: Ipv4Addr,
    multicast_ipv6: Ipv6Addr,

    config: Config
}

impl Options {
    fn initialise_args<'a>() -> ArgMatches<'a> {
        App::new(PROJECT_NAME)
            .arg(Arg::with_name(OPTION_CONFIG_FILE)
                .short("c")
                .long(OPTION_CONFIG_FILE)
                .value_name("FILE")
                .takes_value(true))
            .arg(Arg::with_name(OPTION_DEVICE_ID)
                .short("d")
                .long(OPTION_DEVICE_ID)
                .takes_value(true))
            .arg(Arg::with_name(OPTION_DEVICE_NAME)
                .short("n")
                .long(OPTION_DEVICE_NAME)
                .takes_value(true))
            .arg(Arg::with_name(OPTION_USE_HOSTNAME)
                .short("u")
                .long(OPTION_USE_HOSTNAME)
                .takes_value(false))
            .arg(Arg::with_name(OPTION_PORT)
                .short("p")
                .long(OPTION_PORT)
                .takes_value(true))
            .arg(Arg::with_name(OPTION_MULTICAST_IPV4)
                .short("i")
                .long(OPTION_MULTICAST_IPV4)
                .takes_value(true))
            .arg(Arg::with_name(OPTION_MULTICAST_IPV6)
                .short("a")
                .long(OPTION_MULTICAST_IPV6)
                .takes_value(true))
            .get_matches()
    }

    pub fn new() -> Self {
        let app = Self::initialise_args();
        let config_file = match app.value_of_os(OPTION_CONFIG_FILE) {
            None => get_config_path(),
            Some(value) => Some(PathBuf::from(value))
        };
        let config = match config_file {
            None => Config::default(),
            Some(path) => Config::new(&path)
        };
        Options {
            device_id: match app.value_of(OPTION_DEVICE_ID) {
                None => config.device_id.clone().unwrap_or_else(|| Uuid::new_v4()),
                Some(value) => Options::validate_type_or_exit(value, "an id")
            },
            device_name: match app.value_of(OPTION_DEVICE_NAME) {
                None => config.device_name.clone().unwrap_or_else(|| OsString::from(Uuid::new_v4().to_string())),
                Some(value) => Options::validate_type_or_exit(value, "a valid name")
            },
            use_hostname: if app.is_present(OPTION_USE_HOSTNAME) { true } else { config.use_hostname.clone().unwrap_or_else(|| true) },
            port: match app.value_of(OPTION_PORT) {
                None => config.port.clone().unwrap_or_else(|| 11529),
                Some(value) => Options::validate_type_or_exit(value, "a port")
            },
            multicast_ipv4: match app.value_of(OPTION_MULTICAST_IPV4) {
                None => config.multicast_ipv4.clone().unwrap_or_else(|| Ipv4Addr::new(244, 0, 0, 134)),
                Some(value) => Options::validate_type_or_exit(value, "a IPv4 address")
            },
            multicast_ipv6: match app.value_of(OPTION_MULTICAST_IPV6) {
                None => config.multicast_ipv6.clone().unwrap_or_else(|| Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0134)),
                Some(value) => Options::validate_type_or_exit(value, "a IPv6 address")
            },
            config
        }
    }

    fn validate_type_or_exit<T: FromStr>(value: &str, message: &str) -> T {
        match T::from_str(value) {
            Ok(value) => value,
            Err(_) => {
                clap::Error {
                    message: format!("error: Invalid value: unable to parse {} as {}", value, message),
                    kind: clap::ErrorKind::InvalidValue,
                    info: None
                }.exit();
            }
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    device_id: Option<Uuid>,
    device_name: Option<OsString>,
    use_hostname: Option<bool>,
    port: Option<u16>,
    multicast_ipv4: Option<Ipv4Addr>,
    multicast_ipv6: Option<Ipv6Addr>
}

impl Config {
    fn deserialize(config: &String) -> Self {
        match toml::from_str::<Config>(&config) {
            Ok(config) => config,
            Err(e) => {
                warn!("Unable to deserialize config: {}", e);
                Self::default()
            }
        }
    }

    fn serialize(&self) -> Option<String> {
        let serialized = match toml::to_string(&self) {
            Ok(id) => id,
            Err(e) => {
                warn!("Failed to serialize Config: {}", e);
                return None;
            }
        };
        Some(serialized)
    }

    fn new(file_path: &PathBuf) -> Self {
        let mut config = match read_to_string(file_path) {
            Ok(config) => Self::deserialize(&config),
            Err(e) => {
                warn!("Unable to read config file: {}", e);
                Self::default()
            }
        };

        // Value should be saved
        config.set_device_id();
        config.write_to_file(file_path);
        config
    }

    fn write_to_file(&self, file_path: &PathBuf) {
        if let Some(x) = self.serialize() {
            write(file_path, x).unwrap_or_else(|e| warn!("Unable to write config: {}", e));
        };
    }

    fn set_device_id(&mut self) {
        self.device_id.get_or_insert_with(|| {
            let new_id = Uuid::new_v4();
            info!("Created new device id: {}", new_id);
            new_id
        });
    }
}

fn get_config_path() -> Option<PathBuf> {
    if let Some(project_dir) = ProjectDirs::from("", "", PROJECT_NAME) {
        if !project_dir.config_dir().exists() {
            create_dir_all(project_dir.config_dir())
                .unwrap_or_else(|e| warn!("Unable to create default directory: {}", e));
        }
        let mut config_dir = project_dir.config_dir().to_path_buf();
        config_dir.push(CONFIG_FILE);
        return Some(config_dir);
    }
    warn!("No valid home directory!");
    None
}
