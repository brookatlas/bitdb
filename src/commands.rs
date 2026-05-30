use crate::types::{self, BitobaseObject};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

pub fn handle_select_command(
    command: &types::RedisCommand,
    _: &Arc<Mutex<HashMap<String, BitobaseObject>>>,
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
    _: &Arc<Mutex<HashMap<String, BitobaseObject>>>,
) -> Result<String, String> {
    return Ok(format!("+PONG\r\n"));
}

pub fn handle_get_command(
    command: &types::RedisCommand,
    db: &Arc<Mutex<HashMap<String, BitobaseObject>>>,
) -> Result<String, String> {
    if command.args.len() != 1 {
        return Ok(format!(
            "-ERR wrong number of arguments for 'get' command\r\n"
        ));
    }
    {
        let data = db.lock().unwrap();
        let key = command.args[0].clone();
        if let Some(value) = data.get(&key) {
            return Ok(format!("+{}\r\n", value.to_string()));
        } else {
            return Ok(String::from("+(nil)\r\n"));
        }
    }
}

pub fn handle_set_command(
    command: &types::RedisCommand,
    db: &Arc<Mutex<HashMap<String, BitobaseObject>>>,
) -> Result<String, String> {
    if command.args.len() != 2 {
        return Ok(format!(
            "-ERR wrong number of arguments for 'set' command\r\n"
        ));
    }
    {
        let mut data = db.lock().unwrap();
        let key = command.args[0].clone();
        let value = command.args[1].clone();
        let object: BitobaseObject = BitobaseObject::String(value);
        data.insert(key, object);
    }

    return Ok(String::from("+OK\r\n"));
}

pub fn handle_incr_command(
    command: &types::RedisCommand,
    db: &Arc<Mutex<HashMap<String, BitobaseObject>>>,
) -> Result<String, String> {
    if command.args.len() != 1 {
        return Ok(format!(
            "-ERR wrong number of arguments for 'incr' command\r\n"
        ));
    }
    {
        let mut data = db.lock().unwrap();
        let key = command.args[0].clone();
        if let Some(value) = data.get(&key) {
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
            data.insert(key, object);
        } else {
            data.insert(key, BitobaseObject::String(String::from("1")));
        }
    }

    return Ok(String::from("+OK\r\n"));
}

pub fn handle_lpush_command(
    command: &types::RedisCommand,
    db: &Arc<Mutex<HashMap<String, BitobaseObject>>>,
) -> Result<String, String> {
    if command.args.len() < 2 {
        return Ok(format!(
            "-ERR wrong number of arguments for 'lpush' command\r\n"
        ));
    }
    {
        let mut data = db.lock().unwrap();
        let key = command.args[0].clone();
        let values = &command.args[1..];

        if matches!(data.get(&key), Some(BitobaseObject::String(_))) {
            return Ok(String::from("-ERR expected list but found string.\r\n"));
        }
        let object = data
            .entry(key)
            .or_insert_with(|| BitobaseObject::List(VecDeque::new()));
        if let BitobaseObject::List(l) = object {
            for value in values.iter() {
                l.push_front(value.to_string());
            }
        }
        return Ok(format!("+Ok \r\n"));
    }
}

pub fn handle_rpush_command(
    command: &types::RedisCommand,
    db: &Arc<Mutex<HashMap<String, BitobaseObject>>>,
) -> Result<String, String> {
    if command.args.len() < 2 {
        return Ok(format!(
            "-ERR wrong number of arguments for 'rpush' command\r\n"
        ));
    }
    {
        let mut data = db.lock().unwrap();
        let key = command.args[0].clone();
        let values = &command.args[1..];

        if matches!(data.get(&key), Some(BitobaseObject::String(_))) {
            return Ok(String::from("-ERR expected list but found string.\r\n"));
        }
        let object = data
            .entry(key)
            .or_insert_with(|| BitobaseObject::List(VecDeque::new()));
        if let BitobaseObject::List(l) = object {
            for value in values.iter() {
                l.push_back(value.to_string());
            }
        }
        return Ok(format!("+Ok \r\n"));
    }
}
