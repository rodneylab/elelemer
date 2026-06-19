#[cfg(test)]
pub mod test_helpers;

use std::io::{self, BufWriter, Write};

use clap::Parser;

use elelemer::{
    cli::Cli,
    configuration::{AppSettings, RunnerSettings},
    errors::AppError,
    runners::{Runner, llamacpp::LlamaCppRunner, ollama::OllamaRunner},
};

#[cfg(feature = "internal-tools")]
use elelemer::docs;

async fn run_command<W: Write>(writer: &mut W, settings: AppSettings) -> Result<(), AppError> {
    #[cfg(feature = "internal-tools")]
    debug_assert!(!matches!(command, Commands::MarkdownHelp));

    match settings.runner {
        RunnerSettings::Llamacpp(runner_settings) => {
            Runner::<LlamaCppRunner>::new(LlamaCppRunner::new(), &runner_settings)?
                .invoke(
                    writer,
                    &settings.system_prompt,
                    &settings.model,
                    &settings.prompt,
                )
                .await
        }
        RunnerSettings::Ollama(runner_settings) => {
            Runner::<OllamaRunner>::new(OllamaRunner::new(), &runner_settings)?
                .invoke(
                    writer,
                    &settings.system_prompt,
                    &settings.model,
                    &settings.prompt,
                )
                .await
        }
    }
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    #[cfg(feature = "internal-tools")]
    if matches!(cli.command, Commands::MarkdownHelp) {
        docs::write_markdown_docs_to_file::<Cli>("docs/help.md")?;

        return Ok(());
    }

    cli.initialise_logging();
    log::info!("🐘 elelemer");

    let stdout = io::stdout();
    let mut handle = BufWriter::new(stdout.lock());

    let app_settings = AppSettings::load(cli)?;

    Ok(run_command(&mut handle, app_settings).await?)
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use elelemer::configuration::{ApiConfig, AppSettings, RunnerSettings, UserApiConfig};
    use secrecy::SecretString;

    use crate::{run_command, test_helpers::LOGGING};

    #[tokio::test]
    async fn run_command_returns_error_when_api_is_unreachable() {
        // arrange
        let log_capture = LazyLock::force(&LOGGING);
        let mut writer = Vec::<u8>::new();
        let settings = AppSettings {
            runner: RunnerSettings::Llamacpp(
                ApiConfig::try_from(UserApiConfig {
                    base_url: String::from("https://ollama.example.com"),
                    api_key: SecretString::from("not-needed"),
                    timeout_secs: 1,
                })
                .unwrap(),
            ),
            system_prompt: String::from("You are a helpful assistant"),
            model: String::from("qxz-supreme-sota"),
            prompt: String::from("How hot is the sun?"),
        };

        // act
        let outcome = run_command(&mut writer, settings).await.unwrap_err();

        // assert
        assert_eq!(
            outcome.to_string(),
            "Connection error while sending request to llama.cpp server"
        );
        let logs = log_capture.logs();
        assert!(
            &logs.contains(
                "WARN — request returned an error: transport error: client error (Connect)"
            )
        );
    }
}
