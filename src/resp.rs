use crate::commands::Command;
use anyhow::{Error, Result};
use std::str;

const CRLF: &[u8] = "\r\n".as_bytes();

#[derive(Debug)]
pub enum RESPType {
    Null,
    Error(String),
    Integer(u64),
    BulkString(String),
    Array(Vec<RESPType>),
    SimpleString(String),
}

fn take_until_crlf(bytes: &[u8]) -> usize {
    let mut n = 0;
    while &(bytes[n..n + 2]) != CRLF {
        n += 1;
    }
    return n;
}

impl RESPType {
    pub fn pack(self: &Self) -> Vec<u8> {
        let mut result: Vec<u8> = vec![];
        match self {
            Self::BulkString(s) => {
                result.push(b'$');
                let mut str_bytes = s.as_bytes().to_owned();
                let mut length_bytes = str_bytes.len().to_string().as_bytes().to_owned();
                result.append(&mut length_bytes);
                result.append(&mut CRLF.clone().to_owned());
                result.append(&mut str_bytes);
                result.append(&mut CRLF.clone().to_owned());
            }
            Self::SimpleString(s) => {
                result.push(b'+');
                let mut str_bytes = s.as_bytes().to_owned();
                result.append(&mut str_bytes);
                result.append(&mut CRLF.clone().to_owned());
            }
            Self::Error(s) => {
                result.push(b'-');
                let mut str_bytes = s.as_bytes().to_owned();
                result.append(&mut str_bytes);
                result.append(&mut CRLF.clone().to_owned());
            }
            Self::Integer(i) => {
                result.push(b':');
                let mut str_bytes = i.to_string().as_bytes().to_owned();
                result.append(&mut str_bytes);
                result.append(&mut CRLF.clone().to_owned());
            }
            Self::Array(a) => {
                result.push(b'*');
                let mut str_bytes = a.len().to_string().as_bytes().to_owned();
                result.append(&mut str_bytes);
                result.append(&mut CRLF.clone().to_owned());
                for item in a {
                    result.append(&mut item.pack());
                }
            }
            Self::Null => {
                result.push(b'$');
                result.append(&mut "-1".as_bytes().to_owned());
                result.append(&mut CRLF.clone().to_owned());
            }
        }

        return result;
    }
    pub fn unpack(bytes: &[u8]) -> (Self, usize) {
        match bytes[0] {
            b'+' => {
                let n = take_until_crlf(&bytes[1..]);
                return (
                    RESPType::SimpleString(str::from_utf8(&bytes[1..n + 1]).unwrap().to_string()),
                    n + 3,
                );
            }
            b'-' => {
                let n = take_until_crlf(&bytes[1..]);
                return (
                    RESPType::Error(str::from_utf8(&bytes[1..n + 1]).unwrap().to_string()),
                    n + 3,
                );
            }
            b':' => {
                let n = take_until_crlf(&bytes[1..]);
                return (
                    RESPType::Integer(str::from_utf8(&bytes[1..n + 1]).unwrap().parse().unwrap()),
                    n + 3,
                );
            }
            b'$' => {
                let len_len = take_until_crlf(&bytes[1..]);
                let len: usize = str::from_utf8(&bytes[1..len_len + 1])
                    .unwrap()
                    .parse()
                    .unwrap();
                return (
                    RESPType::BulkString(
                        str::from_utf8(&bytes[len_len + 3..len_len + 3 + len])
                            .unwrap()
                            .to_string(),
                    ),
                    len_len + 3 + len + 2,
                );
            }
            b'*' => {
                let len_len = take_until_crlf(&bytes[1..]);
                let num_elements: usize = str::from_utf8(&bytes[1..len_len + 1])
                    .unwrap()
                    .parse()
                    .unwrap();
                let mut result: Vec<RESPType> = vec![];
                let mut used_length_in_elements = 0;
                let header_size = 1 + len_len + 2;
                for _ in 0..num_elements {
                    let (element, used_size) =
                        RESPType::unpack(&bytes[header_size + used_length_in_elements..]);
                    result.push(element);
                    used_length_in_elements += used_size
                }

                return (
                    RESPType::Array(result),
                    header_size + used_length_in_elements,
                );
            }
            _ => {
                return (RESPType::Error("Invalid RESP type".to_string()), 0);
            }
        }
    }
    pub fn pack_string(self: &Self) -> Result<&str> {
        match self {
            Self::BulkString(s) => Ok(s),
            Self::SimpleString(s) => Ok(s),
            _ => Err(Error::msg("Trying to decode non-string")),
        }
    }
    pub fn to_command(self: &Self) -> Result<Command> {
        match self {
            Self::Array(elements) => {
                if let RESPType::BulkString(command) = elements.get(0).unwrap() {
                    return Command::new(
                        &command,
                        elements.into_iter().skip(1).collect::<Vec<_>>(),
                    );
                }

                Err(Error::msg("not a command"))
            }
            _ => Err(Error::msg("not an array")),
        }
    }
}
