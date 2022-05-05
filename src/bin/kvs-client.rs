use anyhow::{Ok, Result};
use clap::{arg, Command};
use kvs::proc;
use log::error;
use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
};
use env_logger::Builder;
use env_logger::Target;
use std::process::exit;

fn main() -> Result<()> {
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stderr);
    env_logger::init();
    let matches = Command::new("kvs-client")
        .version("0.1.0")
        .author("nick <yxiao196@gmail.com>")
        .about("kv client")
        .subcommand(
            Command::new("get")
                .about("get value of key")
                .arg(arg!([KEY]))
                .arg(arg!(--"addr" <ADDR>).required(false)),
        )
        .subcommand(
            Command::new("set")
                .about("set key and value")
                .arg(arg!([KEY]))
                .arg(arg!([VALUE]))
                .arg(arg!(--"addr" <ADDR>).required(false)),
        )
        .subcommand(
            Command::new("rm")
                .about("rm by key")
                .arg(arg!([KEY]))
                .arg(arg!(--"addr" <ADDR>).required(false)),
        )
        .get_matches();

    // addr
    let mut socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4000);

    let mut cmd = String::new();
    match matches.subcommand() {
        Some(("get", sub)) => {
            if let Some(addr) = sub.value_of("addr") {
                socket = addr.parse::<SocketAddr>()?;
            }
            let mut stream = TcpStream::connect(socket)?;
            cmd.push_str("get");
            cmd.push_str(" ");
            cmd.push_str(sub.value_of("KEY").unwrap());
            cmd.push_str("\r\n");
            stream.write(cmd.as_bytes())?;
            let mut buf = [0; 1024];
            stream.read(&mut buf)?;
            let s = std::str::from_utf8(&buf)?;
            let res = proc::deserialize(s.to_string())?;
            print!("{}", res);
        }
        Some(("set", sub)) => {
            if let Some(addr) = sub.value_of("addr") {
                socket = addr.parse::<SocketAddr>()?;
            }
            let mut stream = TcpStream::connect(socket)?;
            cmd.push_str("set");
            cmd.push_str(" ");
            cmd.push_str(sub.value_of("KEY").unwrap());
            cmd.push_str(" ");
            cmd.push_str(sub.value_of("VALUE").unwrap());
            cmd.push_str("\r\n");
            stream.write(cmd.as_bytes())?;
        }
        Some(("rm", sub)) => {
            if let Some(addr) = sub.value_of("addr") {
                socket = addr.parse::<SocketAddr>()?;
            }
            let mut stream = TcpStream::connect(socket)?;
            cmd.push_str("rm");
            cmd.push_str(" ");
            cmd.push_str(sub.value_of("KEY").unwrap());
            cmd.push_str("\r\n");
            stream.write(cmd.as_bytes())?;
            let mut buf = [0; 1024];
            stream.read(&mut buf)?;
            let s = std::str::from_utf8(&buf)?;
            let res = proc::deserialize(s.to_string())?;
            if res != "Ok\n" {
                error!("{}", res);
                exit(1);
            }
        }
        _ => {
            error!("need command");
            exit(1);
        },
    }

    Ok(())
}
