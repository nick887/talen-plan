use crate::error::KvStoreError;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::SeekFrom;
use std::io::{BufRead, BufReader, LineWriter, Seek, Write};
use std::string::String;
use std::time::SystemTime;
use std::{collections::HashMap, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
enum Operation {
    Set,
    Get,
    Rm,
}
#[derive(Serialize, Deserialize, Debug)]
struct Entry {
    timestamp: SystemTime,
    key_size: u64,
    value_size: u64,
    key: String,
    value: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Command {
    op: Operation,
    key: Option<String>,
    value: Option<String>,
}

pub struct KvStore {
    _map: HashMap<String, String>,
    // r_file: File,
    file: File,
}

impl KvStore {
    // pub fn new() -> KvStore {
    //     KvStore {
    //         map: HashMap::new(),
    //         path: String::from(""),
    //     }
    // }
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let mut p = path.into();
        println!("{:?}",p.as_path());

        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&p)?;

        println!("yes");
        if f.metadata()?.is_dir() {
            p.push(r"/log");
            println!("{:?}",p.as_path());
            let f = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&p)?;
        }

        Ok(KvStore {
            _map: HashMap::new(),
            file: f,
        })
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.file.seek(SeekFrom::End(0))?;
        let mut writer = LineWriter::new(&self.file);
        let s = serde_json::to_string(&Command {
            op: Operation::Set,
            key: Some(key),
            value: Some(value),
        })?;
        writer.write(&s.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.get_key(key) {
            Err(err) => {
                return Err(err);
            }
            Ok(cmd) => match cmd {
                Some(cmd) => Ok(cmd.value),
                None => return Ok(None),
            },
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        match self.get_key(key) {
            Err(err) => {
                return Err(err);
            }
            Ok(cmd) => match cmd {
                Some(mut cmd) => {
                    self.file.seek(SeekFrom::End(0))?;
                    let mut writer = LineWriter::new(&self.file);
                    cmd.op = Operation::Rm;
                    let s = serde_json::to_string(&cmd)?;
                    writer.write(s.as_bytes())?;
                    writer.write(b"\n")?;
                    return Ok(());
                }
                None => {
                    return Err(KvStoreError::NotFoundKey)?;
                }
            },
        }
    }

    fn get_key(&mut self, key: String) -> Result<Option<Command>> {
        let reader = BufReader::new(&self.file);
        for line in reader.lines() {
            let line = line.unwrap();
            let cmd: Command = serde_json::from_str(&line)?;
            let k1 = cmd.key.clone().unwrap();
            if key == k1 {
                return Ok(Some(cmd));
            }
        }
        Ok(None)
    }
}
