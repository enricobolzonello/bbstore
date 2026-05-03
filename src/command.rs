use crate::errors::ProtocolError;
use clap::{Args, Subcommand};
use std::{fmt, str::FromStr};

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
    command: ConfigCommand,
}

#[derive(Subcommand, Debug)]
enum ConfigCommand {
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

impl FromStr for Command {
    type Err = ProtocolError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let words: Vec<&str> = s.splitn(3, ' ').collect();
        match words.as_slice() {
            [] | [""] => Err(ProtocolError::EmptyCommand),
            [cmd, args @ ..] => match *cmd {
                "GET" => match args {
                    [key] if !key.is_empty() => Ok(Command::Get {
                        key: key.to_string(),
                    }),
                    [_] => Err(ProtocolError::RequiredArguments {
                        command: "GET".into(),
                        arguments: "key".into(),
                    }),
                    _ => Err(ProtocolError::InvalidNumberOfArguments {
                        command: "GET".into(),
                        expected: 1,
                        received: args.len(),
                    }),
                },
                "SET" => match args {
                    [key, value] if !key.is_empty() && !value.is_empty() => Ok(Command::Set {
                        key: key.to_string(),
                        value: value.to_string(),
                    }),
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
                "CONFIG" => match args {
                    ["REWRITE"] => Ok(Command::Config(ConfigArgs {
                        command: ConfigCommand::Rewrite,
                    })),
                    [sub] => Err(ProtocolError::UnknownSubcommand {
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
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_get() {
        let cmd: Command = "GET mykey".parse().unwrap();
        assert!(matches!(cmd, Command::Get { key } if key == "mykey"));
    }

    #[test]
    fn from_str_set() {
        let cmd: Command = "SET mykey myvalue".parse().unwrap();
        assert!(matches!(cmd, Command::Set { key, value } if key == "mykey" && value == "myvalue"));
    }

    #[test]
    fn from_str_set_value_with_spaces() {
        let cmd: Command = "SET mykey hello world".parse().unwrap();
        assert!(
            matches!(cmd, Command::Set { key, value } if key == "mykey" && value == "hello world")
        );
    }

    #[test]
    fn from_str_config_rewrite() {
        let cmd: Command = "CONFIG REWRITE".parse().unwrap();
        assert!(matches!(cmd, Command::Config(_)));
    }

    #[test]
    fn from_str_unknown_command() {
        assert!("DELETE mykey".parse::<Command>().is_err());
    }

    #[test]
    fn from_str_unknown_config_subcommand() {
        assert!("CONFIG GET maxmemory".parse::<Command>().is_err());
    }

    #[test]
    fn from_str_empty_input() {
        assert!("".parse::<Command>().is_err());
    }

    #[test]
    fn from_str_get_missing_key() {
        assert!("GET".parse::<Command>().is_err());
    }

    #[test]
    fn from_str_set_missing_value() {
        assert!("SET mykey".parse::<Command>().is_err());
    }

    #[test]
    fn from_str_get_empty_key() {
        let err = "GET ".parse::<Command>().unwrap_err();
        assert!(matches!(err, ProtocolError::RequiredArguments { ref command, .. } if command == "GET"));
    }

    #[test]
    fn from_str_get_extra_argument() {
        let err = "GET mykey extra".parse::<Command>().unwrap_err();
        assert!(matches!(err, ProtocolError::InvalidNumberOfArguments { expected: 1, .. }));
    }

    #[test]
    fn from_str_set_no_args() {
        let err = "SET".parse::<Command>().unwrap_err();
        assert!(matches!(err, ProtocolError::InvalidNumberOfArguments { ref command, .. } if command == "SET"));
    }

    #[test]
    fn from_str_set_empty_key() {
        let err = "SET ".parse::<Command>().unwrap_err();
        assert!(matches!(err, ProtocolError::InvalidNumberOfArguments { ref command, .. } if command == "SET"));
    }

    #[test]
    fn from_str_config_no_subcommand() {
        let err = "CONFIG".parse::<Command>().unwrap_err();
        assert!(matches!(err, ProtocolError::RequiredArguments { ref command, .. } if command == "CONFIG"));
    }

    #[test]
    fn display_roundtrip_get() {
        let original = "GET mykey";
        assert_eq!(original.parse::<Command>().unwrap().to_string(), original);
    }

    #[test]
    fn display_roundtrip_set() {
        let original = "SET mykey myvalue";
        assert_eq!(original.parse::<Command>().unwrap().to_string(), original);
    }

    #[test]
    fn display_roundtrip_set_value_with_spaces() {
        let original = "SET mykey hello world";
        assert_eq!(original.parse::<Command>().unwrap().to_string(), original);
    }

    #[test]
    fn display_roundtrip_config_rewrite() {
        let original = "CONFIG REWRITE";
        assert_eq!(original.parse::<Command>().unwrap().to_string(), original);
    }
}
