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
            ["GET", key] => Ok(Command::Get {
                key: key.to_string(),
            }),
            ["SET", key, value] => Ok(Command::Set {
                key: key.to_string(),
                value: value.to_string(),
            }),
            ["CONFIG", "REWRITE"] => Ok(Command::Config(ConfigArgs {
                command: ConfigCommand::Rewrite,
            })),
            ["CONFIG", cmd, ..] => bail!("Unknown CONFIG subcommand {}", cmd),
            [cmd, ..] => bail!("Unknown command {}", cmd),
            [] => bail!("Empty command"),
        }
    }
}
