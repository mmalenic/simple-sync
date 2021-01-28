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

const CONFIG_FILE: &'static str = "config.toml";

macro_rules! options_string {
    ($name:expr) => {
        str::replace(stringify!($name).split(".").last().unwrap_or_default(), "_", "-")
    }
}

macro_rules! merge_with {
    ($( #[ $attr:meta ] )*
    pub struct $name:ident {
        $(
            $( #[ $field_attr:meta ] )*
            $field_name:ident : $field_type:ty $( , )?
        )*
    }) => {
        $( #[ $attr ] )*
        pub struct $name {
            $(
                $( #[$field_attr] )*
                $field_name : $field_type,
            )*
        }

        impl $name {
            fn merge_with(&mut self, other: $name, matches: &ArgMatches) {
                $(if !matches.is_present(options_string!(self.$field_name)) {
                    self.$field_name = other.$field_name;
                })*
            }
        }
    }
}

lazy_static! {
    static ref DEVICE_ID: String = Uuid::new_v4().to_string();
    static ref DEVICE_NAME: OsString = get_hostname();
}

merge_with! {
    #[derive(Debug, StructOpt, Serialize, Deserialize)]
    #[serde(default, rename_all = "kebab-case")]
    pub struct Options {
        // Options
        #[structopt(long, short = "c", value_name("FILE"))]
        #[serde(skip)]
        config_file: Option<PathBuf>,

        #[structopt(long, short = "i", default_value = &DEVICE_ID)]
        #[serde(skip_serializing)]
        device_id: Uuid,

        #[structopt(long, short = "n", default_value_os = &DEVICE_NAME)]
        set_device_name: OsString,

        #[structopt(long, short = "p", default_value = "11529")]
        #[serde(skip_serializing)]
        port: u16,

        #[structopt(long, short = "h", default_value = "244.0.0.134")]
        #[serde(skip_serializing)]
        multicast_ipv4: Ipv4Addr,

        #[structopt(long, short = "H", default_value = "ff02::134")]
        #[serde(skip_serializing)]
        multicast_ipv6: Ipv6Addr,

        // Flags
        #[structopt(long, short = "N")]
        #[serde(skip_serializing)]
        no_config_file: bool,
    }
}

impl Options {
    fn from_args_with_conf(path: &PathBuf) -> Self {
        let from_conf = Self::from_conf(path);
        let mut from_args = Self::from_args();
        let matches = Self::clap().get_matches();

        from_args.merge_with(from_conf, &matches);
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
    }

    fn write_to_file(&self, file_path: &PathBuf) {
        if let Some(x) = self.serialize() {
            write(file_path, x).unwrap_or_else(|e| warn!("Unable to write config: {}", e));
        };
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
