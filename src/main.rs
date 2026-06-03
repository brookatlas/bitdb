use crate::types::BitobaseObject;
use dashmap::DashMap;
use std::{env, sync::Arc};
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

mod commands;
mod resp;
pub mod types;

#[tokio::main]
async fn main() {
    let run_arguments = get_run_arguments();
    let listener = match TcpListener::bind(&run_arguments.listen_url).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind: {}: {}", run_arguments.listen_url, e);
            std::process::exit(1);
        }
    };
    let db: Arc<DashMap<String, BitobaseObject>> = Arc::new(DashMap::new());
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                stream.set_nodelay(true).unwrap();
                let db_clone = Arc::clone(&db);
                tokio::spawn(async move {
                    let _ = handle_client(stream, &db_clone).await;
                });
            }
            Err(e) => eprintln!("Accepted connection error: {e}"),
        }
    }
}

fn get_run_arguments() -> types::BitobaseRunArguments {
    let listen_url = env::var("LISTEN_URL").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    types::BitobaseRunArguments { listen_url }
}

async fn handle_client(
    stream: TcpStream,
    db: &Arc<DashMap<String, BitobaseObject>>,
) -> Result<(), String> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    loop {
        let resp_message = match resp::parse_resp_message(&mut reader).await {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("PARSE ERROR: {:?}", e);
                return Err(e);
            }
        };
        let redis_command = resp::resp_message_to_redis_command(&resp_message)?;
        let redis_response: String = match redis_command.command.to_lowercase().as_str() {
            "select" => commands::handle_select_command(&redis_command, db)?,
            "ping" => commands::handle_ping_command(&redis_command, db)?,
            "get" => commands::handle_get_command(&redis_command, db)?,
            "set" => commands::handle_set_command(&redis_command, db)?,
            "mset" => commands::handle_mset_command(&redis_command, db)?,
            "incr" => commands::handle_incr_command(&redis_command, db)?,
            "lpush" => commands::handle_lpush_command(&redis_command, db)?,
            "rpush" => commands::handle_rpush_command(&redis_command, db)?,
            "lpop" => commands::handle_lpop_command(&redis_command, db)?,
            "rpop" => commands::handle_rpop_command(&redis_command, db)?,
            "config" => commands::handle_config_command(&redis_command, db)?,
            _ => format!("-ERR unknown command '{}'\r\n", redis_command.command),
        };
        if let Err(e) = writer.write_all(redis_response.as_bytes()).await {
            eprintln!("Failed to write to client: {}", e);
            return Err(e.to_string());
        }
    }
}
