use dashmap::DashMap;
use std::{
    env,
    io::{BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use crate::types::BitobaseObject;

mod commands;
mod resp;
pub mod types;

fn main() {
    let run_arguments = get_run_arguments();
    let listener = match TcpListener::bind(&run_arguments.listen_url) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind: {}: {}", run_arguments.listen_url, e);
            std::process::exit(1);
        }
    };
    let db: Arc<DashMap<String, BitobaseObject>> = Arc::new(DashMap::new());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                stream.set_nodelay(true).unwrap();
                let db_clone = Arc::clone(&db);
                thread::spawn(move || {
                    let _ = handle_client(&stream, &db_clone);
                });
            }
            Err(e) => {
                eprintln!("Accepted connection error: {e}")
            }
        }
    }
}

fn get_run_arguments() -> types::BitobaseRunArguments {
    let listen_url_key = "LISTEN_URL";
    let listen_url = env::var(listen_url_key).unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let run_arguments = types::BitobaseRunArguments {
        listen_url: listen_url,
    };
    return run_arguments;
}

fn handle_client(
    stream: &TcpStream,
    db: &Arc<DashMap<String, BitobaseObject>>,
) -> Result<String, String> {
    let mut reader = BufReader::new(stream);
    loop {
        let resp_message: Vec<String> = match resp::parse_resp_message(&mut reader) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("PARSE ERROR: {:?}", e);
                return Err(e);
            }
        };
        let redis_command: types::RedisCommand =
            resp::resp_message_to_redis_command(&resp_message)?;
        let mut s = stream;
        let redis_response: String = match redis_command.command.to_lowercase().as_str() {
            "select" => commands::handle_select_command(&redis_command, &db)?,
            "ping" => commands::handle_ping_command(&redis_command, &db)?,
            "get" => commands::handle_get_command(&redis_command, &db)?,
            "set" => commands::handle_set_command(&redis_command, &db)?,
            "mset" => commands::handle_mset_command(&redis_command, &db)?,
            "incr" => commands::handle_incr_command(&redis_command, &db)?,
            "lpush" => commands::handle_lpush_command(&redis_command, db)?,
            "rpush" => commands::handle_rpush_command(&redis_command, db)?,
            "lpop" => commands::handle_lpop_command(&redis_command, db)?,
            "rpop" => commands::handle_rpop_command(&redis_command, db)?,
            "config" => commands::handle_config_command(&redis_command, db)?,
            _ => format!("-ERR unknown command '{}'\r\n", redis_command.command),
        };
        if let Err(e) = s.write_all(redis_response.as_bytes()) {
            eprintln!("Failed to write to client: {}", e);
        }
    }
}
