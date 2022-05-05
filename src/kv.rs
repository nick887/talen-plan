use crate::error::KvStoreError;
use crate::kv_engine::KvsEngine;
use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::fs::{File, OpenOptions};
use std::io::SeekFrom;
use std::io::{BufRead, BufReader, LineWriter, Seek, Write};
use std::string::String;
use std::time::{SystemTime, UNIX_EPOCH};
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
    file: File,
    map: HashMap<String, u64>,
    last_compaction_time: u64,
    path: PathBuf,
}

pub struct SledEngine {
    db: Db,
}

impl SledEngine {
    pub fn open(path: impl Into<PathBuf>) -> Result<SledEngine> {
        let p = path.into();
        info!("dir of log file: {:?}", p);
        let db = sled::open(&p)?;
        Ok(SledEngine { db })
    }
}

impl KvsEngine for SledEngine {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        self.db.insert(key.as_str(), value.as_str())?;
        self.db.flush()?;
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        let val = self.db.get(&key)?;
        match val {
            Some(val) => {
                let mut x = val.to_vec();
                let res = x.as_mut_slice();
                let res = std::str::from_utf8(res)?.to_string();
                Ok(Some(res))
            }
            None => Ok(None),
        }
    }

    // remove the key in map and append log
    fn remove(&mut self, key: String) -> Result<()> {
        let val = self.db.remove(key.as_str())?;
        self.db.flush()?;
        match val {
            Some(_) => {
                Ok(())
            },
            None => {
                Err(KvStoreError::NotFoundKey)?
            }
        }
    }
}

impl KvsEngine for KvStore {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        self.update(Command {
            op: Operation::Set,
            key: Some(key),
            value: Some(value),
        })?;
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
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

    // remove the key in map and append log
    fn remove(&mut self, key: String) -> Result<()> {
        match self.get_key(key) {
            Err(err) => {
                return Err(err);
            }
            Ok(cmd) => match cmd {
                Some(cmd) => {
                    // cmd must be like set key val
                    self.update(Command {
                        op: Operation::Rm,
                        key: cmd.key,
                        value: None,
                    })?;
                    Ok(())
                }
                None => Err(KvStoreError::NotFoundKey)?,
            },
        }
    }
}

impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let mut p = path.into();
        let mut f = File::open(&p)?;
        if f.metadata()?.is_dir() {
            p.push(r"log");
            f = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&p)?;
        } else {
            f = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&p)?;
        }

        let map = KvStore::file_map(&mut f)?;
        Ok(KvStore {
            path: p,
            file: f,
            map,
            last_compaction_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    // compaction on condition
    pub fn compaction(&mut self) -> Result<()> {
        if SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - self.last_compaction_time
            > 10
        {
            let mut all_map = HashMap::new();
            for (key, value) in self.map.iter() {
                let mut buf = String::new();
                self.file.seek(SeekFrom::Start(*value))?;
                let mut reader = BufReader::new(&mut self.file);
                reader.read_line(&mut buf)?;
                let cmd: Command = serde_json::from_str(&buf)?;
                all_map.insert(key.clone(), cmd);
            }
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&self.path)?;
            // to the start of file
            self.file.seek(SeekFrom::Start(0))?;
            let mut pos = 0;
            let mut writer = LineWriter::new(&mut self.file);
            for (key, value) in all_map.into_iter() {
                let s = serde_json::to_string(&value)?;
                writer.write(s.as_bytes())?;
                writer.write(&[0xA])?;
                self.map.insert(key, pos);
                pos += s.bytes().len() as u64 + 1;
            }
            self.last_compaction_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        Ok(())
    }

    // write the append log and update map
    fn update(&mut self, cmd: Command) -> Result<()> {
        // println!("{:?}", cmd);
        self.compaction()?;
        let pos = self.file.seek(SeekFrom::End(0))?;
        let mut writer = LineWriter::new(&mut self.file);
        let s = serde_json::to_string(&cmd)?;
        match cmd.op {
            Operation::Set => {
                writer.write(s.as_bytes())?;
                self.map.insert(cmd.key.unwrap(), pos);
            }
            Operation::Rm => {
                writer.write(s.as_bytes())?;
                self.map.remove(&cmd.key.unwrap());
            }
            _ => {}
        }
        writer.write(&[0xA])?;
        Ok(())
    }

    // get pos from map and read command from file
    fn get_key(&mut self, key: String) -> Result<Option<Command>> {
        if self.map.contains_key(&key) {
            let cmd = KvStore::get_cmd(&mut self.file, *self.map.get(&key).unwrap())?;
            return Ok(Some(cmd));
        } else {
            return Ok(None);
        }
    }

    // read command from file
    fn get_cmd(file: &mut File, pos: u64) -> Result<Command> {
        file.seek(SeekFrom::Start(pos))?;
        let mut reader = BufReader::new(file);
        let mut buf = String::new();
        reader.read_line(&mut buf)?;
        let cmd = serde_json::from_str(&buf)?;
        Ok(cmd)
    }

    // read entire log file and flush to map
    fn file_map(file: &mut File) -> Result<HashMap<String, u64>> {
        let mut map: HashMap<String, u64> = HashMap::new();
        file.seek(SeekFrom::Start(0))?;
        let mut reader = BufReader::new(file);
        loop {
            let mut buf = String::new();
            let n = reader.read_line(&mut buf)?;
            if n == 0 {
                break;
            }

            let cmd: Command = serde_json::from_str(&buf)?;
            match cmd.op {
                Operation::Set => {
                    let pos = reader.stream_position()?;

                    map.insert(cmd.key.unwrap(), pos - buf.as_bytes().len() as u64);
                }
                Operation::Rm => {
                    map.remove(&cmd.key.unwrap());
                }
                _ => {}
            }
        }
        Ok(map)
    }
}
