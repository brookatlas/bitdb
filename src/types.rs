pub struct BitobaseRunArguments {
    pub listen_url: String,
}

pub struct RedisCommand {
    pub command: String,
    pub args: Vec<String>,
}
