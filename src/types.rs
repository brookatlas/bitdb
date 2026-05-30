use std::collections::VecDeque;

pub struct BitobaseRunArguments {
    pub listen_url: String,
}

pub struct RedisCommand {
    pub command: String,
    pub args: Vec<String>,
}

pub enum BitobaseObject {
    String(String),
    List(VecDeque<String>),
}

impl BitobaseObject {
    pub fn to_string(&self) -> String {
        match self {
            BitobaseObject::String(s) => format!("\"{}\"", s),
            BitobaseObject::List(l) => {
                let mut items: Vec<String> = Vec::new();
                for item in l.iter() {
                    items.push(item.clone());
                }
                let result = format!("[{}]", items.join(","));
                return format!("{}", result);
            }
        }
    }
}
