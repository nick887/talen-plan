use anyhow::Result;
use clap::{arg, Command};
use kvs::kv_engine::KvsEngine;
use kvs::KvStore;
use std::process::exit;

fn main() -> Result<()> {
    let matches = Command::new("kvs")
        .version("0.1.0")
        .author("nick <yxiao196@gmail.com>")
        .about("to store key and value")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("get")
                .about("get value of key")
                .arg(arg!([KEY])),
        )
        .subcommand(
            Command::new("set")
                .about("set key and value")
                .arg(arg!([KEY]))
                .arg(arg!([VALUE])),
        )
        .subcommand(Command::new("rm").about("rm by key").arg(arg!([KEY])))
        .get_matches();

    let mut kvs = Box::new(KvStore::open(".")?);
    match matches.subcommand() {
        Some(("get", sub)) => match kvs.get(sub.value_of("KEY").unwrap().to_string()) {
            Err(_err) => {
                //  println!("{}",err);
                exit(1);
            }
            Ok(val) => match val {
                Some(val) => {
                    println!("{}", val);
                    exit(0);
                }
                None => {
                    println!("Key not found");
                    exit(0);
                }
            },
        },
        Some(("set", sub)) => match kvs.set(
            sub.value_of("KEY").unwrap().to_string(),
            sub.value_of("VALUE").unwrap().to_string(),
        ) {
            // eprintln!("unimplemented");
            Err(err) => {
                println!("{}", err);
                exit(1);
            }
            Ok(_) => {
                exit(0);
            }
        },
        Some(("rm", sub)) => match kvs.remove(sub.value_of("KEY").unwrap().to_string()) {
            Err(err) => {
                // println!("Key not found");
                println!("{}", err);
                exit(1);
            }
            Ok(_) => {
                exit(0);
            }
        },
        _ => println!("error"),
    }
    panic!();
}
