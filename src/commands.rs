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
    pub fn new(name: &str, args: &[&RESPType]) -> Result<Self> {
        match name.to_lowercase().as_str() {
            "ping" => {
                let msg = args.get(0).map(|arg| arg.pack_string());

                match msg {
                    Some(Ok(value)) => Ok(Self::Ping(Some(value.to_string()))),
                    Some(Err(_)) => Err(Error::msg("Invalid value")),
                    None => Ok(Self::Ping(None)),
                }
            }
            "get" => {
                let key = args.get(0).map(|arg| arg.pack_string());

                match key {
                    Some(Ok(key)) => Ok(Self::Get(key.to_string())),
                    _ => Err(Error::msg("Invalid key")),
                }
            }
            "echo" => {
                let key = args.get(0).map(|arg| arg.pack_string());

                match key {
                    Some(Ok(key)) => Ok(Self::Echo(key.to_string())),
                    _ => Err(Error::msg("Invalid message")),
                }
            }
            "set" => {
                let key = args.get(0).map(|arg| arg.pack_string());
                let value = args.get(1).map(|arg| arg.pack_string());
                let px = args.get(3).map(|arg| arg.pack_string());

                match (key, value) {
                    (Some(Ok(key)), Some(Ok(value))) => Ok(Self::Set {
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
            Self::Echo(msg) => RESPType::BulkString(msg.clone()),
            Self::Ping(data) => data.clone().map_or_else(
                || RESPType::SimpleString("PONG".to_string()),
                RESPType::BulkString,
            ),
            Self::Get(key) => cache.get(key).map_or(RESPType::Null, RESPType::BulkString),
            Self::Set { key, value, px } => {
                cache.set(key.clone(), value.clone(), *px);

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
            Self::Get(key) => write!(f, "GET {key}"),
            Self::Echo(value) => write!(f, "ECHO {value}"),
            Self::Ping(value) => match value {
                Some(value) => write!(f, "PING {value}"),
                None => write!(f, "PING"),
            },
            Self::Set { key, value, px } => write!(f, "SET {key} {value} {px:?}"),
        }
    }
}
