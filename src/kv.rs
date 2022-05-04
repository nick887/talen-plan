use crate::error::KvStoreError;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::SeekFrom;
use std::io::{BufRead, BufReader, LineWriter, Seek, Write};
use std::string::String;
use std::time::SystemTime;
use std::{collections::HashMap, path::PathBuf};
const THRESHOLD: usize = 1000;

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
}

impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let mut p = path.into();
        // println!("{:?}",p.as_path());
        let mut f = File::open(&p)?;
        if f.metadata()?.is_dir() {
            p.push(r"log");
            // println!("{:?}", p.as_path());
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
        Ok(KvStore { file: f, map })
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.update(Command {
            op: Operation::Set,
            key: Some(key),
            value: Some(value),
        })?;
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

    // remove the key in map and append log
    pub fn remove(&mut self, key: String) -> Result<()> {
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
    // compaction on condition
    fn compaction(&mut self) -> Result<()> {
        if self.map.len() > THRESHOLD {
            self.file.seek(SeekFrom::Start(0))?;
            let mut writer = LineWriter::new(&mut self.file);
            for (key, value) in self.map.into_iter() {
                let s = serde_json::to_string(&Command {
                    op: Operation::Set,
                    key: Some(key),
                    value: Some(value),
                })?;
                writer.write(buf)
            }
        } else {
        }
        Ok(())
    }

    // write the append log and update map
    fn update(&mut self, cmd: Command) -> Result<()> {
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
