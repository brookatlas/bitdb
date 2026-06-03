use std::future::Future;
use std::pin::Pin;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;

use crate::types;

pub fn resp_message_to_redis_command(
    resp_message: &Vec<String>,
) -> Result<types::RedisCommand, String> {
    let mut command = types::RedisCommand {
        command: String::new(),
        args: vec![],
    };
    for (index, message) in resp_message.iter().enumerate() {
        if index == 0 {
            command.command = message.clone();
        } else {
            command.args.push(message.clone());
        }
    }
    Ok(command)
}

// Recursive async fns need Box::pin to avoid infinite future size
pub fn parse_resp_message(
    reader: &mut BufReader<OwnedReadHalf>,
) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>> + Send + '_>> {
    Box::pin(async move {
        let mut buf = [0u8; 1];
        let mut result: Vec<String> = vec![];
        match reader.read_exact(&mut buf).await {
            Ok(_) => {
                let c = buf[0] as char;
                match c {
                    '*' => {
                        let mut array_message = parse_resp_array(reader).await?;
                        result.append(&mut array_message);
                    }
                    '+' => {
                        let message = parse_resp_simple_string(reader).await?;
                        result.push(message);
                    }
                    '-' => {
                        let message = parse_resp_simple_error(reader).await?;
                        return Err(message);
                    }
                    '$' => {
                        let message = parse_resp_bulk_string(reader).await?;
                        result.push(message);
                    }
                    _ => {
                        let mut line = String::new();
                        reader
                            .read_line(&mut line)
                            .await
                            .map_err(|e| e.to_string())?;
                        let full = format!(
                            "{}{}",
                            c,
                            line.trim_end_matches("\r\n").trim_end_matches("\n")
                        );
                        let parts: Vec<String> =
                            full.split_whitespace().map(String::from).collect();
                        result.extend(parts);
                    }
                }
            }
            Err(e) => return Err(e.to_string()),
        }
        Ok(result)
    })
}

pub async fn parse_resp_simple_string(
    reader: &mut BufReader<OwnedReadHalf>,
) -> Result<String, String> {
    let mut string = String::new();
    reader
        .read_line(&mut string)
        .await
        .map_err(|e| format!("error reading simple string: {}", e))?;
    Ok(string.trim_end_matches("\r\n").to_string())
}

pub async fn parse_resp_simple_error(
    reader: &mut BufReader<OwnedReadHalf>,
) -> Result<String, String> {
    let mut string = String::new();
    reader
        .read_line(&mut string)
        .await
        .map_err(|e| format!("error reading error line: {}", e))?;
    Ok(string.trim_end_matches("\r\n").to_string())
}

pub async fn parse_resp_array(
    reader: &mut BufReader<OwnedReadHalf>,
) -> Result<Vec<String>, String> {
    let mut buffer = String::new();
    reader
        .read_line(&mut buffer)
        .await
        .map_err(|e| format!("error reading array size: {}", e))?;
    let size = buffer.trim().parse::<i32>().map_err(|e| e.to_string())?;
    let mut message_v: Vec<String> = vec![];
    for _ in 0..size {
        let mut message_result = parse_resp_message(reader).await?;
        message_v.append(&mut message_result);
    }
    Ok(message_v)
}

pub async fn parse_resp_bulk_string(
    reader: &mut BufReader<OwnedReadHalf>,
) -> Result<String, String> {
    let mut size_buffer = String::new();
    let mut string_buffer = String::new();
    reader
        .read_line(&mut size_buffer)
        .await
        .map_err(|e| format!("error reading bulk string size: {}", e))?;
    reader
        .read_line(&mut string_buffer)
        .await
        .map_err(|e| format!("error reading bulk string: {}", e))?;
    Ok(string_buffer.trim_end_matches("\r\n").to_string())
}

pub fn resp_array(items: &[&str]) -> String {
    let mut resp = format!("*{}\r\n", items.len());
    for item in items {
        resp.push_str(&format!("${}\r\n{}\r\n", item.len(), item));
    }
    resp
}
