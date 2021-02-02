use std::ffi::OsString;
use std::fs::{create_dir_all, read_to_string, write, File};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;
use std::str::FromStr;

use clap::{App, Arg, ArgMatches};
use directories::ProjectDirs;
use log::{info, warn};
use serde::{Deserialize, Serialize, Serializer};
use uuid::Uuid;
use structopt::StructOpt;
use lazy_static::lazy_static;

use crate::PROJECT_NAME;
use std::io::{BufReader, BufRead, Error};
use itertools::Itertools;
use serde::ser::SerializeStruct;

const CONFIG_FILE: &'static str = "config.toml";
const PROGRAM_DATA: &'static str = "data.toml";

lazy_static! {
    static ref DEVICE_ID: String = Uuid::new_v4().to_string();
    static ref DEVICE_NAME: OsString = get_hostname();
}

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
        #[serde(alias = "device_name")]
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
    fn from_args_with_conf() -> Self {
        let mut from_args = Self::from_args();

        if from_args.no_config_file {
            return from_args
        }

        let from_conf = match from_args.config_file {
            None => match get_config_path() {
                None => Self::default(),
                Some(mut path) => {
                    path.push(CONFIG_FILE);
                    Self::from_conf(&path)
                }
            }
            Some(ref path) => Self::from_conf(path)
        };

        let matches = Self::clap().get_matches();
        from_args.merge_with(from_conf, &matches);
        from_args
    }

    fn deserialize_options(options: &str) -> Self {
        match toml::from_str::<Options>(&options) {
            Ok(config) => config,
            Err(e) => {
                warn!("Unable to deserialize config: {}", e);
                Self::default()
            }
        }
    }

    fn serialize_options(&self) -> Option<String> {
        let serialized = match toml::to_string(&self) {
            Ok(id) => id,
            Err(e) => {
                warn!("Failed to serialize Config: {}", e);
                return None;
            }
        };
        Some(serialized)
    }

    fn from_conf(path: &PathBuf) -> Self {
        let file = Self::read_conf(path);
        if file.is_empty() {
            Self::default()
        } else {
            Self::deserialize_options(&file)
        }
    }

    fn read_conf(path: &PathBuf) -> String {
        match read_to_string(path) {
            Ok(file) => file,
            Err(e) => {
                warn!("Unable to read config file: {}", e);
                ""
            }.to_string()
        }
    }

    fn write_to_file(&mut self, path: &PathBuf) {
        if let Some(x) = self.serialize_options() {
            write(path, x).unwrap_or_else(|e| warn!("Unable to write config: {}", e));
        };
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
