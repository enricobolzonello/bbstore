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
