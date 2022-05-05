use crate::kv::Command;
use crate::kv::Operation;
use anyhow::Result;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Seek;
use std::io::SeekFrom;

// read command from file
pub fn get_cmd(file: &mut File, pos: u64) -> Result<Command> {
    file.seek(SeekFrom::Start(pos))?;
    let mut reader = BufReader::new(file);
    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    let cmd = serde_json::from_str(&buf)?;
    Ok(cmd)
}

// read entire log file and flush to map
pub fn file_map(file: &mut File) -> Result<HashMap<String, u64>> {
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
