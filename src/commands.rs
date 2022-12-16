use crate::{cache::Cache, resp::RESPType};
use anyhow::{Error, Result};
use std::fmt;

pub enum Command {
    Get(String),
    Echo(String),
    Ping(Option<String>),
    Set {
        key: String,
        value: String,
        px: Option<u64>,
    },
}

impl Command {
    pub fn new<'a>(name: &'a str, args: Vec<&RESPType>) -> Result<Command> {
        match name.to_lowercase().as_str() {
            "ping" => {
                let msg = args.get(0).map(|arg| arg.pack_string());

                match msg {
                    Some(Ok(value)) => Ok(Command::Ping(Some(value.to_string()))),
                    Some(Err(_)) => Err(Error::msg("Invalid value")),
                    None => Ok(Command::Ping(None)),
                }
            }
            "get" => {
                let key = args.get(0).map(|arg| arg.pack_string());

                match key {
                    Some(Ok(key)) => Ok(Command::Get(key.to_string())),
                    _ => Err(Error::msg("Invalid key")),
                }
            }
            "echo" => {
                let key = args.get(0).map(|arg| arg.pack_string());

                match key {
                    Some(Ok(key)) => Ok(Command::Echo(key.to_string())),
                    _ => Err(Error::msg("Invalid message")),
                }
            }
            "set" => {
                let key = args.get(0).map(|arg| arg.pack_string());
                let value = args.get(1).map(|arg| arg.pack_string());
                let px = args.get(3).map(|arg| arg.pack_string());

                match (key, value) {
                    (Some(Ok(key)), Some(Ok(value))) => Ok(Command::Set {
                        key: key.to_string(),
                        value: value.to_string(),
                        px: match px {
                            Some(Ok(px)) => Some(px.parse::<u64>().unwrap()),
                            _ => None,
                        },
                    }),
                    _ => Err(Error::msg("Invalid key or value")),
                }
            }
            _ => Err(Error::msg("Unsupported command")),
        }
    }

    pub fn run(&self, cache: &mut Cache) -> RESPType {
        match self {
            Command::Echo(msg) => RESPType::BulkString(msg.clone()),
            Command::Ping(data) => match data {
                Some(msg) => RESPType::BulkString(msg.clone()),
                None => RESPType::SimpleString("PONG".to_string()),
            },
            Command::Get(key) => match cache.get(&key) {
                Some(value) => RESPType::BulkString(value),
                None => RESPType::Null,
            },
            Command::Set { key, value, px } => {
                cache.set(key.clone(), value.clone(), px.clone());

                RESPType::SimpleString("OK".to_string())
            }
            #[allow(unreachable_patterns)] // Redis has more commands than the ones we support
            _ => RESPType::Error("Unsupported command".to_string()),
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Get(key) => write!(f, "GET {}", key),
            Command::Echo(value) => write!(f, "ECHO {}", value),
            Command::Ping(value) => match value {
                Some(value) => write!(f, "PING {}", value),
                None => write!(f, "PING"),
            },
            Command::Set { key, value, px } => write!(f, "SET {} {} {:?}", key, value, px),
        }
    }
}
