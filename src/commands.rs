use crate::{
    resp::resp_array,
    types::{self, BitobaseObject},
};
use dashmap::DashMap;
use std::{collections::VecDeque, sync::Arc};

pub fn handle_select_command(
    command: &types::RedisCommand,
    _: &Arc<DashMap<String, BitobaseObject>>,
) -> Result<String, String> {
    if command.args.len() < 1 {
        return Ok(format!(
            "-ERR wrong number of arguments for 'select' command\r\n"
        ));
    }
    return Ok(format!("+OK\r\n"));
}

pub fn handle_ping_command(
    _: &types::RedisCommand,
    _: &Arc<DashMap<String, BitobaseObject>>,
) -> Result<String, String> {
    return Ok(format!("+PONG\r\n"));
}

pub fn handle_get_command(
    command: &types::RedisCommand,
    db: &Arc<DashMap<String, BitobaseObject>>,
) -> Result<String, String> {
    if command.args.len() != 1 {
        return Ok(format!(
            "-ERR wrong number of arguments for 'get' command\r\n"
        ));
    }
    {
        let key = command.args[0].as_str();
        if let Some(value) = db.get(key) {
            return Ok(format!("+{}\r\n", value.to_string()));
        } else {
            return Ok(String::from("+(nil)\r\n"));
        }
    }
}

pub fn handle_set_command(
    command: &types::RedisCommand,
    db: &Arc<DashMap<String, BitobaseObject>>,
) -> Result<String, String> {
    if command.args.len() != 2 {
        return Ok(format!(
            "-ERR wrong number of arguments for 'set' command\r\n"
        ));
    }
    {
        let key = command.args[0].clone();
        let value = command.args[1].clone();
        let object: BitobaseObject = BitobaseObject::String(value);
        db.insert(key, object);
    }

    return Ok(String::from("+OK\r\n"));
}

pub fn handle_mset_command(
    command: &types::RedisCommand,
    db: &Arc<DashMap<String, BitobaseObject>>,
) -> Result<String, String> {
    if command.args.len() < 2 {
        return Ok(format!(
            "-ERR wrong number of arguments for 'mset' command\r\n"
        ));
    }
    {
        let count = command.args.len();
        for index in 0..(count - 1) {
            let key = command.args[index].clone();
            let value = command.args[index + 1].clone();
            let object: BitobaseObject = BitobaseObject::String(value);
            db.insert(key, object);
        }
    }

    return Ok(String::from("+OK\r\n"));
}

pub fn handle_incr_command(
    command: &types::RedisCommand,
    db: &Arc<DashMap<String, BitobaseObject>>,
) -> Result<String, String> {
    if command.args.len() != 1 {
        return Ok(format!(
            "-ERR wrong number of arguments for 'incr' command\r\n"
        ));
    }
    {
        let key = command.args[0].clone();
        if let Some(value) = db.get(&key) {
            let mut parsed_value = match value.to_string().parse::<i32>() {
                Ok(n) => n,
                Err(_) => {
                    return Ok(String::from(
                        "-ERR value is not an integer or out of range\r\n",
                    ));
                }
            };
            parsed_value = parsed_value + 1;
            let object: BitobaseObject = BitobaseObject::String(parsed_value.to_string());
            db.insert(key, object);
        } else {
            db.insert(key, BitobaseObject::String(String::from("1")));
        }
    }

    return Ok(String::from("+OK\r\n"));
}

pub fn handle_lpush_command(
    command: &types::RedisCommand,
    db: &Arc<DashMap<String, BitobaseObject>>,
) -> Result<String, String> {
    if command.args.len() < 2 {
        return Ok(format!(
            "-ERR wrong number of arguments for 'lpush' command\r\n"
        ));
    }
    {
        let key = command.args[0].clone();
        let values = &command.args[1..];

        if db
            .get(&key)
            .map(|e| matches!(e.value(), BitobaseObject::String(_)))
            .unwrap_or(false)
        {
            return Ok(String::from("-ERR expected list but found string.\r\n"));
        }
        let mut object = db
            .entry(key)
            .or_insert_with(|| BitobaseObject::List(VecDeque::new()));
        if let BitobaseObject::List(l) = object.value_mut() {
            for value in values.iter() {
                l.push_front(value.to_string());
            }
        }
        return Ok(format!("+Ok \r\n"));
    }
}

pub fn handle_rpush_command(
    command: &types::RedisCommand,
    db: &Arc<DashMap<String, BitobaseObject>>,
) -> Result<String, String> {
    if command.args.len() < 2 {
        return Ok(format!(
            "-ERR wrong number of arguments for 'rpush' command\r\n"
        ));
    }
    {
        let key = command.args[0].clone();
        let values = &command.args[1..];

        if db
            .get(&key)
            .map(|e| matches!(e.value(), BitobaseObject::String(_)))
            .unwrap_or(false)
        {
            return Ok(String::from("-ERR expected list but found string.\r\n"));
        }
        let mut object = db
            .entry(key)
            .or_insert_with(|| BitobaseObject::List(VecDeque::new()));
        if let BitobaseObject::List(l) = object.value_mut() {
            for value in values.iter() {
                l.push_back(value.to_string());
            }
        }
        return Ok(format!("+Ok \r\n"));
    }
}

pub fn handle_lpop_command(
    command: &types::RedisCommand,
    db: &Arc<DashMap<String, BitobaseObject>>,
) -> Result<String, String> {
    if command.args.len() < 2 {
        return Ok(format!(
            "-ERR wrong number of arguments for 'lpop' command\r\n"
        ));
    } else {
        let key = command.args[0].clone();
        let count = command.args[1]
            .as_str()
            .parse::<i32>()
            .map_err(|_| "count was not a integer")?;
        match db.get_mut(&key) {
            Some(mut entry) => {
                match entry.value_mut() {
                    BitobaseObject::List(l) => {
                        for _ in 0..count {
                            l.pop_front();
                        }
                        return Ok("+OK\r\n".to_string());
                    }
                    BitobaseObject::String(_) => {
                        return Ok("-WRONGTYPE Operation against a key holding the wrong kind of value\r\n".to_string());
                    }
                }
            }
            None => {
                return Ok("$-1\r\n".to_string());
            }
        }
    }
}

pub fn handle_rpop_command(
    command: &types::RedisCommand,
    db: &Arc<DashMap<String, BitobaseObject>>,
) -> Result<String, String> {
    if command.args.len() < 2 {
        return Ok(format!(
            "-ERR wrong number of arguments for 'rpop' command\r\n"
        ));
    } else {
        let key = command.args[0].clone();
        let count = command.args[1]
            .as_str()
            .parse::<i32>()
            .map_err(|_| "count was not a integer")?;
        match db.get_mut(&key) {
            Some(mut entry) => match entry.value_mut() {
                BitobaseObject::List(l) => {
                    for _ in 0..count {
                        l.pop_back();
                    }
                    return Ok("+OK\r\n".to_string());
                }
                BitobaseObject::String(_) => {
                    return Ok(format!("-ERR wrong data type for 'rpop' command"));
                }
            },
            None => {
                return Ok("+(nil)\r\n".to_string());
            }
        }
    }
}

pub fn handle_config_command(
    command: &types::RedisCommand,
    db: &Arc<DashMap<String, BitobaseObject>>,
) -> Result<String, String> {
    if command.args.len() < 1 {
        return Ok(format!(
            "-ERR wrong number of arguments for 'config' command\r\n"
        ));
    }
    let config_command = command.args[0].as_str();
    if config_command.to_lowercase() != "get" {
        return Ok(format!(
            "-ERR subcommand {} in 'config' command is not supported\r\n",
            config_command
        ));
    }
    return handle_config_get_command(command, db);
}

pub fn handle_config_get_command(
    command: &types::RedisCommand,
    _: &Arc<DashMap<String, BitobaseObject>>,
) -> Result<String, String> {
    match command.args[1].as_str() {
        "appendonly" => {
            let message_array: [&str; 2] = ["appendonly", "no"];
            let message = resp_array(&message_array);
            return Ok(message);
        }
        "save" => {
            let message_array: [&str; 2] = ["save", ""];
            let message = resp_array(&message_array);
            return Ok(message);
        }
        _ => {
            return Ok(format!("*0\r\n"));
        }
    }
}
