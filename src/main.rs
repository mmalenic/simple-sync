mod config;

#[macro_use]
extern crate derive_serialize_into;
extern crate serde;

use std::net::{Ipv6Addr, Ipv4Addr};
use structopt::StructOpt;
use uuid::Uuid;
use std::ffi::OsString;

const PROJECT_NAME: &str = "simple-simple-sync";

fn main() {
    let ip = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0134);
    println!("{}", ip);
}