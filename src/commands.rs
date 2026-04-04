use crate::types;

pub fn handle_get_command(command: &types::RedisCommand) -> Result<String, String> {
    return Ok(format!("-ERR unknown command: {}\r\n", command.command));
}

pub fn handle_set_command(command: &types::RedisCommand) -> Result<String, String> {
    return Ok(format!("-ERR unknown command: {}\r\n", command.command));
}
