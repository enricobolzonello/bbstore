#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("empty command")]
    EmptyCommand,
    #[error("{command:?} requires {arguments:?}")]
    RequiredArguments { command: String, arguments: String },
    #[error("{command:?} requires exactly {expected:?}, received {received:?}")]
    InvalidNumberOfArguments {
        command: String,
        expected: usize,
        received: usize,
    },
    #[error("unknown {command:?} subcommand {subcommand:?}")]
    UnknownSubcommand { command: String, subcommand: String },
    #[error("unknown command {0}")]
    UnknownCommand(String),
}
