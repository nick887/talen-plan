use anyhow::Result;
use clap::{arg, Command};
use env_logger::Builder;
use env_logger::Target;
use kvs::kv::SledEngine;
use kvs::proc;
use kvs::{kv_engine, KvStore};
use log::error;
use log::info;
use std::fs;
use std::io::Write;
use std::process::exit;
use std::{
    io::Read,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
};
enum Engine {
    Sled,
    Kv,
    NotDefined,
}

fn main() -> Result<()> {
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stderr);
    env_logger::init();
    let matches = Command::new("kvs-server")
        .version("0.1.0")
        .author("nick <yxiao196@gmail.com>")
        .about("stateful server to store key and value")
        .arg(arg!(--addr <ADDR>).required(false))
        .arg(arg!(--engine <ENGINE>).required(false))
        .get_matches();

    let mut kv_engine: Box<dyn kv_engine::KvsEngine>;
    let mut engine_type: Engine = Engine::Sled;
    let mut dir_empty = true;
    // verify the engine is same
    for entry in fs::read_dir(".")? {
        let dir = entry?;
        if dir.path().as_path().to_str().unwrap() == "./log" {
            dir_empty = false;
            println!("{:?}",dir.path().to_str().unwrap());
            engine_type = Engine::Kv;
        }else if dir.path().as_path().to_str().unwrap() == "./db" {
            dir_empty = false;
        }
    }
    if dir_empty {
        engine_type = Engine::NotDefined;
    }
    // engine
    if let Some(engine) = matches.value_of("engine") {
        if engine == "sled" {
            match engine_type {
                Engine::Kv => {
                    error!("not same engine");
                    exit(1);
                }
                _ => {}
            }
            // sled
            kv_engine = Box::new(SledEngine::open(".")?);
        } else {
            match engine_type {
                Engine::Sled => {
                    error!("not same engine");
                    exit(1);
                }
                _ => {}
            }
            kv_engine = Box::new(KvStore::open(".")?);
        }
    } else {
        kv_engine = Box::new(KvStore::open(".")?);
    }
    // addr
    let mut socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4000);
    if let Some(addr) = matches.value_of("addr") {
        socket = addr.parse::<SocketAddr>()?;
    }

    let listener = TcpListener::bind(socket)?;
    info!("server started at host: {}", socket.to_string());
    info!("CARGO_PKG_VERSION: {}", env!("CARGO_PKG_VERSION"));
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream, &mut kv_engine)?;
    }
    Ok(())
}

// set key val
// get key
// rm key
fn handle_connection(mut stream: TcpStream, kv: &mut Box<dyn kv_engine::KvsEngine>) -> Result<()> {
    let mut buf = [0; 1024];
    // let mut buf = Vec::new();
    stream.read(&mut buf)?;
    let s = std::str::from_utf8(&buf)?;
    info!("cmd: {}", s);
    let s = s
        .split_ascii_whitespace()
        .map(|c| c.to_string())
        .collect::<Vec<String>>();
    if s.len() < 1 {
        return Ok(());
    }
    let mut resp = String::new();
    let cmd_header = s[0].clone();
    if cmd_header == "set" {
        kv.set(s[1].clone(), s[2].clone())?;
    } else if cmd_header == "get" {
        let val = kv.get(s[1].clone())?;
        match val {
            Some(val) => {
                resp = proc::serialize(val, proc::ResPrefix::SIMPLE_STRINGS)?;
            }
            None => {
                resp = proc::serialize(String::from("Key not found"), proc::ResPrefix::ERROR)?;
            }
        }
    } else if cmd_header == "rm" {
        match kv.remove(s[1].clone()) {
            Err(_) => {
                resp = proc::serialize(String::from("Key not found"), proc::ResPrefix::ERROR)?;
            }
            Ok(_) => {
                resp = proc::serialize(String::from("Ok"), proc::ResPrefix::SIMPLE_STRINGS)?;
            }
        }
    }
    stream.write(resp.as_bytes())?;

    Ok(())
}
