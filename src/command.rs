use anyhow::bail;
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
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let words: Vec<&str> = s.splitn(3, ' ').collect();
        match words.as_slice() {
            [] | [""] => bail!("Empty command"),
            ["GET", key] if !key.is_empty() => Ok(Command::Get {
                key: key.to_string(),
            }),
            ["GET"] | ["GET", ""] => bail!("GET requires a key"),
            ["GET", ..] => bail!("GET takes exactly one argument"),
            ["SET", key, value] if !key.is_empty() => Ok(Command::Set {
                key: key.to_string(),
                value: value.to_string(),
            }),
            ["SET"] | ["SET", ""] => bail!("SET requires a key and value"),
            ["SET", _] => bail!("SET requires a value"),
            ["CONFIG", "REWRITE"] => Ok(Command::Config(ConfigArgs {
                command: ConfigCommand::Rewrite,
            })),
            ["CONFIG"] | ["CONFIG", ""] => bail!("CONFIG requires a subcommand"),
            ["CONFIG", sub, ..] => bail!("Unknown CONFIG subcommand: {}", sub),
            [cmd, ..] => bail!("Unknown command: {}", cmd),
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
        assert!(err.to_string().contains("requires a key"));
    }

    #[test]
    fn from_str_get_extra_argument() {
        let err = "GET mykey extra".parse::<Command>().unwrap_err();
        assert!(err.to_string().contains("exactly one argument"));
    }

    #[test]
    fn from_str_set_no_args() {
        let err = "SET".parse::<Command>().unwrap_err();
        assert!(err.to_string().contains("requires a key and value"));
    }

    #[test]
    fn from_str_set_empty_key() {
        let err = "SET ".parse::<Command>().unwrap_err();
        assert!(err.to_string().contains("requires a key and value"));
    }

    #[test]
    fn from_str_config_no_subcommand() {
        let err = "CONFIG".parse::<Command>().unwrap_err();
        assert!(err.to_string().contains("requires a subcommand"));
    }
}
