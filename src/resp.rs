use std::{
    io::{BufRead, BufReader, Read},
    net::TcpStream,
};

use crate::types;

pub fn resp_message_to_redis_command(
    resp_message: &Vec<String>,
) -> Result<types::RedisCommand, String> {
    let mut command: types::RedisCommand = types::RedisCommand {
        command: String::from(""),
        args: vec![],
    };
    for (index, message) in resp_message.iter().enumerate() {
        if index == 0 {
            command.command = String::from(message);
        } else {
            command.args.push(String::from(message));
        }
    }
    return Ok(command);
}

pub fn parse_resp_message(reader: &mut BufReader<&TcpStream>) -> Result<Vec<String>, String> {
    let mut buf = [0u8; 1];
    let mut result: Vec<String> = vec![];
    match reader.read_exact(&mut buf) {
        Ok(_) => {
            let c = buf[0] as char;
            match c {
                '*' => {
                    let mut array_message: Vec<String> = parse_resp_array(reader)?;
                    result.append(&mut array_message);
                }
                '+' => {
                    let message: String = parse_resp_simple_string(reader)?;
                    result.push(message);
                }
                '-' => {
                    let message: String = parse_resp_simple_error(reader)?;
                    return Err(message);
                }
                '$' => {
                    let message: String = parse_resp_bulk_string(reader)?;
                    result.push(message);
                }
                _ => {
                    let mut line = String::new();
                    reader.read_line(&mut line).map_err(|e| e.to_string())?;
                    let full = format!(
                        "{}{}",
                        c,
                        line.trim_end_matches("\r\n").trim_end_matches("\n")
                    );
                    let parts: Vec<String> = full.split_whitespace().map(String::from).collect();
                    result.extend(parts);
                }
            }
        }
        Err(e) => return Err(e.to_string()),
    }
    return Ok(result);
}

pub fn parse_resp_simple_string(reader: &mut BufReader<&TcpStream>) -> Result<String, String> {
    let mut string = String::new();
    if let Err(e) = reader.read_line(&mut string) {
        return Err(format!(
            "error reading simple string line from buffer: {}",
            e.to_string()
        ));
    }
    return Ok(string);
}

pub fn parse_resp_simple_error(reader: &mut BufReader<&TcpStream>) -> Result<String, String> {
    let mut string = String::new();
    if let Err(e) = reader.read_line(&mut string) {
        return Err(format!(
            "error reading error line from buffer: {}",
            e.to_string()
        ));
    }
    return Err(string);
}

pub fn parse_resp_array(reader: &mut BufReader<&TcpStream>) -> Result<Vec<String>, String> {
    let mut buffer = String::new();
    if let Err(e) = reader.read_line(&mut buffer) {
        return Err(format!("error reading line from buffer: {}", e.to_string()));
    }
    let size = buffer
        .as_str()
        .trim()
        .parse::<i32>()
        .map_err(|e| e.to_string())?;
    let mut message_v: Vec<String> = vec![];
    for _ in 0..size as usize {
        let mut message_result = parse_resp_message(reader)?;
        message_v.append(&mut message_result);
    }

    return Ok(message_v);
}

pub fn parse_resp_bulk_string(reader: &mut BufReader<&TcpStream>) -> Result<String, String> {
    let mut size_buffer = String::new();
    let mut string_buffer = String::new();
    if let Err(e) = reader.read_line(&mut size_buffer) {
        return Err(format!("error reading line from buffer: {}", e.to_string()));
    }
    if let Err(e) = reader.read_line(&mut string_buffer) {
        return Err(format!("error reading line from buffer: {}", e.to_string()));
    }
    string_buffer = string_buffer.trim().to_string();

    return Ok(string_buffer);
}

pub fn resp_array(items: &[&str]) -> String {
    let mut resp = format!("*{}\r\n", items.len());
    for item in items {
        resp.push_str(&format!("${}\r\n{}\r\n", item.len(), item));
    }
    return resp;
}
