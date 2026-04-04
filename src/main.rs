use std::{
    env,
    io::{BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
};

mod commands;
mod resp;
pub mod types;

fn main() {
    let run_arguments = get_run_arguments();
    let listener = TcpListener::bind(run_arguments.listen_url).unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(&stream);
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

fn handle_client(stream: &TcpStream) -> Result<String, String> {
    let mut reader = BufReader::new(stream);
    loop {
        let resp_message: Vec<String> = resp::parse_resp_message(&mut reader)?;
        let redis_command: types::RedisCommand =
            resp::resp_message_to_redis_command(&resp_message)?;
        let mut redis_response: String = String::new();
        let mut s = stream;
        match redis_command.command.as_str() {
            "GET" => {
                redis_response = commands::handle_get_command(&redis_command)?;
            }
            "SET" => {
                redis_response = commands::handle_set_command(&redis_command)?;
            }
            _ => {
                redis_response = String::from("-ERR unknown command\r\n");
            }
        }
        if let Err(e) = s.write_all(redis_response.as_bytes()) {
            eprintln!("Failed to write to client: {}", e);
        }
    }
}
