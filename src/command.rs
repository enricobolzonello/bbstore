use crate::{Value, errors::ProtocolError};
use clap::{Args, Subcommand};
use std::fmt;

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(name = "GET")]
    Get { key: String },
    #[command(name = "SET")]
    Set { key: String, value: String },
    #[command(name = "CONFIG")]
    Config(ConfigArgs),
}

#[derive(Args, Debug)]
#[command(args_conflicts_with_subcommands = true)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub(crate) command: ConfigCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum ConfigCommand {
    #[command(name = "REWRITE")]
    Rewrite,
}

impl fmt::Display for ConfigCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigCommand::Rewrite => write!(f, "REWRITE"),
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Get { key } => write!(f, "GET {}", key),
            Command::Set { key, value } => write!(f, "SET {} {}", key, value),
            Command::Config(args) => write!(f, "CONFIG {}", args.command),
        }
    }
}

impl TryFrom<Value> for Command {
    type Error = ProtocolError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let Value::Array(mut args) = value else {
            return Err(ProtocolError::EmptyCommand);
        };

        if args.is_empty() {
            return Err(ProtocolError::EmptyCommand);
        }

        let verb = match args.remove(0) {
            Value::BulkString(s) => s.to_string().to_uppercase(),
            _ => return Err(ProtocolError::EmptyCommand),
        };

        match verb.as_str() {
            "GET" => match args.as_slice() {
                [Value::BulkString(key)] if !key.is_empty() => Ok(Command::Get {
                    key: key.to_string(),
                }),
                [Value::BulkString(_)] => Err(ProtocolError::RequiredArguments {
                    command: "GET".into(),
                    arguments: "key".into(),
                }),
                [] => Err(ProtocolError::RequiredArguments {
                    command: "GET".into(),
                    arguments: "key".into(),
                }),
                _ => Err(ProtocolError::InvalidNumberOfArguments {
                    command: "GET".into(),
                    expected: 1,
                    received: args.len(),
                }),
            },
            "SET" => match args.as_slice() {
                [Value::BulkString(key), Value::BulkString(val)]
                    if !key.is_empty() && !val.is_empty() =>
                {
                    Ok(Command::Set {
                        key: key.to_string(),
                        value: val.to_string(),
                    })
                }
                [_, _] => Err(ProtocolError::RequiredArguments {
                    command: "SET".into(),
                    arguments: "key and value".into(),
                }),
                _ => Err(ProtocolError::InvalidNumberOfArguments {
                    command: "SET".into(),
                    expected: 2,
                    received: args.len(),
                }),
            },
            "CONFIG" => match args.as_slice() {
                [Value::BulkString(sub)] if sub.as_bytes().eq_ignore_ascii_case(b"REWRITE") => {
                    Ok(Command::Config(ConfigArgs {
                        command: ConfigCommand::Rewrite,
                    }))
                }
                [Value::BulkString(sub)] => Err(ProtocolError::UnknownSubcommand {
                    command: "CONFIG".into(),
                    subcommand: sub.to_string(),
                }),
                [] => Err(ProtocolError::RequiredArguments {
                    command: "CONFIG".into(),
                    arguments: "subcommand".into(),
                }),
                _ => Err(ProtocolError::InvalidNumberOfArguments {
                    command: "CONFIG".into(),
                    expected: 1,
                    received: args.len(),
                }),
            },
            cmd => Err(ProtocolError::UnknownCommand(cmd.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arr(args: &[&str]) -> Value {
        Value::Array(
            args.iter()
                .map(|s| Value::BulkString((*s).into()))
                .collect(),
        )
    }

    fn parse(args: &[&str]) -> Result<Command, ProtocolError> {
        Command::try_from(arr(args))
    }

    #[test]
    fn get() {
        let cmd: Command = parse(&["GET", "mykey"]).unwrap();
        assert!(matches!(cmd, Command::Get { key } if key == "mykey"));
    }

    #[test]
    fn set() {
        let cmd: Command = parse(&["SET", "mykey", "myvalue"]).unwrap();
        assert!(matches!(cmd, Command::Set { key, value } if key == "mykey" && value == "myvalue"));
    }

    #[test]
    fn set_value_with_spaces() {
        let cmd = parse(&["SET", "mykey", "hello world"]).unwrap();
        assert!(
            matches!(cmd, Command::Set { key, value } if key == "mykey" && value == "hello world")
        );
    }

    #[test]
    fn config_rewrite() {
        assert!(matches!(
            parse(&["CONFIG", "REWRITE"]).unwrap(),
            Command::Config(_)
        ));
    }

    #[test]
    fn unknown_command() {
        assert!(parse(&["UNKNOWN", "mykey"]).is_err());
    }

    #[test]
    fn unknown_config_subcommand() {
        let err = parse(&["CONFIG", "UNKNOWN"]).unwrap_err();
        assert!(
            matches!(err, ProtocolError::UnknownSubcommand { ref command, .. } if command == "CONFIG")
        );
    }

    #[test]
    fn empty_array() {
        assert!(matches!(
            Command::try_from(arr(&[])).unwrap_err(),
            ProtocolError::EmptyCommand
        ));
    }

    #[test]
    fn get_missing_key() {
        let err = parse(&["GET"]).unwrap_err();
        assert!(
            matches!(err, ProtocolError::RequiredArguments { ref command, .. } if command == "GET")
        );
    }

    #[test]
    fn set_missing_value() {
        let err = parse(&["SET", "mykey"]).unwrap_err();
        assert!(
            matches!(err, ProtocolError::InvalidNumberOfArguments { ref command, .. } if command == "SET")
        );
    }

    #[test]
    fn from_str_get_empty_key() {
        let err = parse(&["GET", ""]).unwrap_err();
        assert!(
            matches!(err, ProtocolError::RequiredArguments { ref command, .. } if command == "GET")
        );
    }

    #[test]
    fn from_str_get_extra_argument() {
        let err = parse(&["GET", "mykey", "extra"]).unwrap_err();
        assert!(matches!(
            err,
            ProtocolError::InvalidNumberOfArguments { expected: 1, .. }
        ));
    }

    #[test]
    fn from_str_set_no_args() {
        let err = parse(&["SET"]).unwrap_err();
        assert!(
            matches!(err, ProtocolError::InvalidNumberOfArguments { ref command, .. } if command == "SET")
        );
    }

    #[test]
    fn from_str_set_empty_key() {
        let err = parse(&["SET", ""]).unwrap_err();
        assert!(
            matches!(err, ProtocolError::InvalidNumberOfArguments { ref command, .. } if command == "SET")
        );
    }

    #[test]
    fn from_str_config_no_subcommand() {
        let err = parse(&["CONFIG"]).unwrap_err();
        assert!(
            matches!(err, ProtocolError::RequiredArguments { ref command, .. } if command == "CONFIG")
        );
    }

    #[test]
    fn display_roundtrip_get() {
        let original = "GET mykey";
        assert_eq!(parse(&["GET", "mykey"]).unwrap().to_string(), original);
    }

    #[test]
    fn display_roundtrip_set() {
        let original = "SET mykey myvalue";
        assert_eq!(
            parse(&["SET", "mykey", "myvalue"]).unwrap().to_string(),
            original
        );
    }

    #[test]
    fn display_roundtrip_set_value_with_spaces() {
        let original = "SET mykey hello world";
        assert_eq!(
            parse(&["SET", "mykey", "hello world"]).unwrap().to_string(),
            original
        );
    }

    #[test]
    fn display_roundtrip_config_rewrite() {
        let original = "CONFIG REWRITE";
        assert_eq!(parse(&["CONFIG", "REWRITE"]).unwrap().to_string(), original);
    }
}
