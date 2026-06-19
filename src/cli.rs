use std::path::PathBuf;

use clap::{Parser, builder::Styles};
use clap_verbosity_flag::Verbosity;

mod styles {
    use clap::builder::styling::{AnsiColor, Effects, Style};

    pub const HEADER: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
    pub const USAGE: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
    pub const LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
    pub const PLACEHOLDER: Style = AnsiColor::Cyan.on_default();
    pub const ERROR: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);

    pub const VALID: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
    pub const INVALID: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);
}

const CARGO_STYLING: Styles = Styles::styled()
    .header(styles::HEADER)
    .usage(styles::USAGE)
    .literal(styles::LITERAL)
    .placeholder(styles::PLACEHOLDER)
    .error(styles::ERROR)
    .valid(styles::VALID)
    .invalid(styles::INVALID);

/// Runner specific arguments
#[derive(Debug, clap::Args)]
pub struct RunArgs {
    /// Model name or alias
    pub model: String,

    /// Runner prompt
    pub prompt: String,

    /// Runner base URL ("http://localhost:11434", for example)
    #[clap(short, long, value_parser)]
    pub base_url: Option<String>,

    /// Runner read timeout in seconds
    #[clap(short, long, value_parser)]
    pub timeout: Option<u64>,

    /// Model system prompt override
    #[clap(short, long, value_parser)]
    pub system_prompt: Option<String>,
}

/// Runner specific commands
#[derive(Debug, clap::Subcommand)]
pub enum RunnerCommands {
    /// Run a model
    Run(RunArgs),
}

/// App commands
#[derive(Debug, clap::Subcommand)]
pub enum Commands {
    /// Use the Ollama large language model runner
    #[command(subcommand)]
    Ollama(RunnerCommands),

    /// Use the llama.cpp large language model runner
    #[command(subcommand)]
    Llamacpp(RunnerCommands),

    /// Generate CLI documentation
    #[cfg(feature = "internal-tools")]
    #[command(hide = true)]
    MarkdownHelp,
}

/// App CLI arguments
#[derive(Parser)]
#[clap(author,version,about,long_about=None)]
#[command(styles=CARGO_STYLING)]
pub struct Cli {
    /// Command
    #[command(subcommand)]
    pub command: Commands,

    /// Path to TOML configuration file.
    #[clap(short, long, value_parser)]
    pub config: Option<PathBuf>,

    /// verbosity -v
    #[clap(flatten)]
    pub verbose: Verbosity,
}

impl Cli {
    /// Configure `env_logger` logging using verbosity flags provided in CLI arguments.
    pub fn initialise_logging(&self) {
        env_logger::Builder::new()
            .filter_level(self.verbose.log_level_filter())
            .init();
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use clap::Parser;

    use crate::cli::Cli;

    /// Parses CLI arguments and returns a `Cli` instance.
    fn parse_args<I, T>(args: I) -> Cli
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        Cli::try_parse_from(args).expect("Failed to parse CLI arguments")
    }

    #[test]
    fn test_no_arguments() {
        // arrange
        let args = vec![
            "program",
            "ollama",
            "run",
            "qxz-supreme-sota:2t",
            "Why is the sky blue, please?",
        ];

        // act
        let cli = parse_args(&args);

        // assert
        assert_eq!(cli.verbose.log_level(), Some(log::Level::Error));
    }

    #[test]
    fn test_verbosity_flag() {
        // arrange
        let args = vec![
            "program",
            "ollama",
            "run",
            "qxz-supreme-sota:2t",
            "-v",
            "Why is the sky blue, please?",
        ];

        // act
        let cli = parse_args(&args);

        // assert
        assert_eq!(cli.verbose.log_level(), Some(log::Level::Warn));
    }

    #[test]
    fn test_verbose_flag_multiple_times() {
        // arrange
        let args = vec![
            "program",
            "ollama",
            "run",
            "qxz-supreme-sota:2t",
            "-vv",
            "Why is the sky blue, please?",
        ];

        // act
        let cli = parse_args(&args);

        // assert
        assert_eq!(cli.verbose.log_level(), Some(log::Level::Info));
    }
}
