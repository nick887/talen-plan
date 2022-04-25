use clap::{arg, Command};
use kvs::KvStore;
use std::process::exit;

fn main() {
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

    let mut kvs = KvStore::new();
    match matches.subcommand() {
        Some(("get", sub)) => match kvs.get(sub.value_of("KEY").unwrap().to_string()) {
            None => {
                eprintln!("unimplemented");
                exit(1);
            }
            Some(val) => {
                println!("{:?}", val);
            }
        },
        Some(("set", sub)) => {
            eprintln!("unimplemented");
            exit(1);
            // kvs.set(sub.value_of("KEY").unwrap().to_string(), sub.value_of("VALUE").unwrap().to_string())
        }
        Some(("rm", sub)) => {
            eprintln!("unimplemented");
            exit(1);
            // kvs.remove(sub.value_of("KEY").unwrap().to_string())
        }
        _ => println!("error"),
    }
}
