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
use structopt::StructOpt;
use lazy_static::lazy_static;

use crate::PROJECT_NAME;
use std::io::Error;

const CONFIG_FILE: &'static str = "config.toml";

macro_rules! options_string {
    ($( $_:ident ).+ $name:ident) => {
        str::replace(stringify!($name), "_", "-")
    }
}

lazy_static! {
    static ref DEVICE_ID: String = Uuid::new_v4().to_string();
    static ref DEVICE_NAME: OsString = get_hostname();
}

fn should_serialize<T>(data: &T) -> bool {
    true
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Options {
    // Options
    #[structopt(long, short = "c", value_name("FILE"))]
    #[serde(skip)]
    config_file: Option<PathBuf>,

    #[structopt(long, short = "i", default_value = &DEVICE_ID)]
    #[serde(skip_serializing_if = "should_serialize")]
    device_id: Uuid,

    #[structopt(long, short = "n", default_value_os = &DEVICE_NAME)]
    device_name: OsString,

    #[structopt(long, short = "p", default_value = "11529")]
    port: u16,

    #[structopt(long, short = "h", default_value = "244.0.0.134")]
    multicast_ipv4: Ipv4Addr,

    #[structopt(long, short = "H", default_value = "ff02::134")]
    multicast_ipv6: Ipv6Addr,

    // Flags
    #[structopt(long, short = "N")]
    no_config_file: bool,

    #[structopt(long, short = "w")]
    #[serde(skip)]
    write_options_to_config: bool
}

impl Options {
    fn from_args_with_conf(path: &PathBuf) -> Self {
        let from_conf = Self::from_conf(path);
        let from_args = Self::from_args();
        let matches = Self::clap().get_matches();

        if !matches.is_present(options_string!(from_args.device_id)) {
            from_args.device_id = from_conf.device_id;
        }
        from_args
    }

    fn deserialize(options: &str) -> Self {
        match toml::from_str::<Options>(&options) {
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

    fn from_conf(file_path: &PathBuf) -> Self {
        match read_to_string(file_path) {
            Ok(config) => Self::deserialize(&config),
            Err(e) => {
                warn!("Unable to read config file: {}", e);
                Self::default()
            }
        }

        // // Value should be saved
        // // config.set_device_id();
        // config.write_to_file(file_path);
        // config
    }

    fn should_serialize(&self) -> bool {
        self.write_options_to_config
    }

    fn write_to_file(&self, file_path: &PathBuf) {
        if let Some(x) = self.serialize() {
            write(file_path, x).unwrap_or_else(|e| warn!("Unable to write config: {}", e));
        };
    }

    fn set_device_id(&mut self, matches: &ArgMatches) {
        if !matches.is_present(options_string!(self.device_id)) && self.device_id.to_string() == *DEVICE_ID {

        }
        // self.device_id.get_or_insert_with(|| {
        //     let new_id = Uuid::new_v4();
        //     info!("Created new device id: {}", new_id);
        //     new_id
        // });
    }

    // pub fn new() -> Self {
    //     let app = Self::initialise_args();
    //     let config_file = match app.value_of_os(OPTION_CONFIG_FILE) {
    //         None => get_config_path(),
    //         Some(value) => Some(PathBuf::from(value))
    //     };
    //     let config = match config_file {
    //         None => Config::default(),
    //         Some(path) => Config::new(&path)
    //     };
    //     Options {
    //         use_hostname: if app.is_present(OPTION_USE_HOSTNAME) { true } else { config.use_hostname.clone().unwrap_or_else(|| true) },
    //
    //         device_id: ,
    //         device_name:,
    //         port: ,
    //         multicast_ipv4: ,
    //         multicast_ipv6: ,
    //
    //         config
    //     }
    // }
    //
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
        Self::from_args()
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

fn get_hostname() -> OsString {
    match hostname::get() {
        Ok(hostname) => hostname,
        Err(error) => {
            warn!("No valid hostname: {}", error);
            OsString::from(Uuid::new_v4().to_string())
        }
    }
}

// #[derive(Serialize, Deserialize, Debug, Default)]
// pub struct Config {
//     device_id: Option<Uuid>,
//     device_name: Option<OsString>,
//     use_hostname: Option<bool>,
//     port: Option<u16>,
//     multicast_ipv4: Option<Ipv4Addr>,
//     multicast_ipv6: Option<Ipv6Addr>
// }
//
// impl Config {
//     fn deserialize(config: &String) -> Self {
//         match toml::from_str::<Config>(&config) {
//             Ok(config) => config,
//             Err(e) => {
//                 warn!("Unable to deserialize config: {}", e);
//                 Self::default()
//             }
//         }
//     }
//
//     fn serialize(&self) -> Option<String> {
//         let serialized = match toml::to_string(&self) {
//             Ok(id) => id,
//             Err(e) => {
//                 warn!("Failed to serialize Config: {}", e);
//                 return None;
//             }
//         };
//         Some(serialized)
//     }
//
//     fn new(file_path: &PathBuf) -> Self {
//         let mut config = match read_to_string(file_path) {
//             Ok(config) => Self::deserialize(&config),
//             Err(e) => {
//                 warn!("Unable to read config file: {}", e);
//                 Self::default()
//             }
//         };
//
//         // Value should be saved
//         config.set_device_id();
//         config.write_to_file(file_path);
//         config
//     }
//
//     fn write_to_file(&self, file_path: &PathBuf) {
//         if let Some(x) = self.serialize() {
//             write(file_path, x).unwrap_or_else(|e| warn!("Unable to write config: {}", e));
//         };
//     }
//
//     fn set_device_id(&mut self) {
//         self.device_id.get_or_insert_with(|| {
//             let new_id = Uuid::new_v4();
//             info!("Created new device id: {}", new_id);
//             new_id
//         });
//     }
// }
//
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     // #[test]
//     // fn test_options() {
//     //     std::env::
//     //     let options = Options::new();
//     //     let
//     // }
// }
