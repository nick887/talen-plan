/*
For Simple Strings, the first byte of the reply is "+"
For Errors, the first byte of the reply is "-"
For Integers, the first byte of the reply is ":"
For Bulk Strings, the first byte of the reply is "$"
For Arrays, the first byte of the reply is "*"
 */
use crate::error::ProcError;
use anyhow::{Ok, Result};

pub enum ResPrefix {
    SIMPLE_STRINGS,
    ERROR,
    INTEGERS,
    BULK_STRINGS,
    ARRAYS,
}

pub fn serialize(content: String, mode: ResPrefix) -> Result<String> {
    let mut res = String::new();
    match mode {
        ResPrefix::SIMPLE_STRINGS => {
            res.push_str("+");
            res.push_str(content.as_str());
        }
        ResPrefix::ERROR => {
            res.push_str("-");
            res.push_str(content.as_str());
        }
        _ => {}
    }
    res.push_str("\r\n");
    Ok(res)
}

pub fn deserialize(origin: String) -> Result<String> {
    if origin.len() < 1 {
        return Err(ProcError::BadLen)?;
    }
    let chars = origin.chars().collect::<Vec<char>>();
    // let header = chars[0];
    let mut s = String::new();
    for char in &chars[1..] {
        if *char == '\r' {
            break;
        }
        s.push(*char);
    }
    s.push('\n');
    return Ok(s);

}
