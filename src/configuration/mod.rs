mod config_path;

#[cfg(test)]
mod serde_ext;

use std::time::Duration;

use config::{ConfigBuilder, builder::DefaultState};
use http::HeaderValue;
use secrecy::{ExposeSecret, SecretString};
use url::Url;

use crate::{
    cli::{Cli, Commands, RunArgs, RunnerCommands},
    configuration::config_path::find_config_file_path,
    errors::{AppError, ConfigError},
};

#[cfg(test)]
pub use crate::configuration::serde_ext::serialise_secret;

const DEFAULT_SYSTEM_PROMPT: &str = "You are a helpful assistant";
const DEFAULT_READ_TIMEOUT: u64 = 3600;
const DEFAULT_OLLAMA_BASE_URL: &str = "http://localhost:11434";
const DEFAULT_OLLAMA_API_KEY: &str = "ollama";
const DEFAULT_LLAMACPP_BASE_URL: &str = "http://localhost:8080";
const DEFAULT_LLAMACPP_API_KEY: &str = "llama.cpp";

/// Runner API configuration
///
/// Used for reading user config.  [`ApiConfig`] is used internally and validates fields.  Two
/// separate struct approach, allows the user to have a configuration, with an invalid url for
/// example, if they are not actually using that particular API;  saving them having meticulously
/// to generate configurations for APIs they do not currently intend to use.
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct UserApiConfig {
    /// Runner API base url
    pub base_url: String,

    /// Runer API key
    #[serde(serialize_with = "serialise_secret")]
    pub api_key: SecretString,

    /// Read timeout, awaiting a runner response
    pub timeout_secs: u64,
}

/// Runner API configuration
///
/// Used internally `base_url` will be a valid URL and `api_key` will be valid as a password in a
/// bearer authorisation HTTP header.
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct ApiConfig {
    /// Runner API base url
    base_url: Url,

    /// Runer API key
    #[serde(serialize_with = "serialise_secret")]
    api_key: SecretString,

    /// Read timeout, awaiting a runner response
    timeout: Duration,
}

impl ApiConfig {
    /// OpenAI-compatible API API key
    pub fn api_key(&self) -> &SecretString {
        &self.api_key
    }

    /// OpenAI-compatible API base url key
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// OpenAI-compatible API read timeout
    pub fn timeout(&self) -> &Duration {
        &self.timeout
    }
}

impl TryFrom<UserApiConfig> for ApiConfig {
    type Error = ConfigError;

    /// # Errors
    /// Errors if:
    /// - UserApiConfig.api_key contains non-visible ASCII characters (characters outside the
    ///   range 32 - 127, both included).
    fn try_from(value: UserApiConfig) -> Result<Self, Self::Error> {
        // API key will be passed as a bearer authorisation header value to the API
        if let Err(_err) =
            HeaderValue::from_str(&format!("Bearer {}", value.api_key.expose_secret()))
        {
            return Err(ConfigError {
                advice: String::from(
                    "Check server API key value is correct; current value does not seem right.",
                ),
                detail: String::from("InvalidHeaderValue"),
            });
        }

        Ok(Self {
            base_url: Url::parse(&value.base_url).map_err(|err| ConfigError {
                advice: format!(
                    "Check server API base url is correct; `{}` does not seem right.",
                    value.base_url
                ),
                detail: format!("Invalid URL: {err}"),
            })?,
            api_key: value.api_key,
            timeout: Duration::from_secs(value.timeout_secs),
        })
    }
}

fn extract_cli_overrides(cli_options: &Cli) -> Result<Vec<(String, config::Value)>, AppError> {
    let mut result = Vec::new();
    if let Commands::Ollama(RunnerCommands::Run(RunArgs {
        base_url, timeout, ..
    })) = &cli_options.command
    {
        if let Some(base_url) = base_url {
            result.push((String::from("ollama.base_url"), base_url.clone().into()));
        }
        if let Some(timeout_secs) = timeout {
            result.push((
                String::from("ollama.timeout"),
                config::Value::from(*timeout_secs),
            ));
        }
    }

    if let Commands::Llamacpp(RunnerCommands::Run(RunArgs {
        base_url, timeout, ..
    })) = &cli_options.command
    {
        if let Some(base_url) = base_url {
            result.push((String::from("llamacpp.base_url"), base_url.clone().into()));
        }
        if let Some(timeout_secs) = timeout {
            result.push((
                String::from("llamacpp.timeout"),
                config::Value::from(*timeout_secs),
            ));
        }
    }

    match cli_options.command {
        Commands::Llamacpp(RunnerCommands::Run(RunArgs {
            ref system_prompt, ..
        }))
        | Commands::Ollama(RunnerCommands::Run(RunArgs {
            ref system_prompt, ..
        })) => {
            if let Some(system_prompt) = system_prompt {
                result.push((String::from("system_prompt"), system_prompt.clone().into()));
            }
        }
        #[cfg(feature = "internal-tools")]
        Commands::MarkdownHelp => {}
    }

    Ok(result)
}

fn apply_cli_overrides(
    cli_options: &Cli,
    config_builder: ConfigBuilder<DefaultState>,
) -> Result<ConfigBuilder<DefaultState>, AppError> {
    let mut builder = config_builder;
    let overrides = extract_cli_overrides(cli_options)?;

    for (path, value) in overrides {
        builder = builder
            .set_override(path, value)
            .map_err(ConfigError::from)?;
    }

    Ok(builder)
}

/// App and runner configuration
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct UserAppSettings {
    /// LLamacpp runner configuration
    pub llamacpp: UserApiConfig,

    /// Ollama runner configuration
    pub ollama: UserApiConfig,

    /// Model system prompt override
    pub system_prompt: String,
}

/// Chat completion parameters and API connection settings for runners
#[derive(Debug)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum RunnerSettings {
    /// Runner settings for the Llama.cpp runner
    Llamacpp(ApiConfig),

    /// Runner settings for the Ollama runner
    Ollama(ApiConfig),
}

/// App and runner configuration
#[derive(Debug)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct AppSettings {
    /// Runner configuration
    pub runner: RunnerSettings,

    /// Model name or alias
    pub model: String,

    /// Runner prompt
    pub prompt: String,

    /// Model system prompt override
    pub system_prompt: String,
}

impl AppSettings {
    fn build_default_config() -> Result<ConfigBuilder<DefaultState>, AppError> {
        Ok(config::Config::builder()
            .set_default("llamacpp.base_url", DEFAULT_LLAMACPP_BASE_URL)
            .map_err(ConfigError::from)?
            .set_default("llamacpp.api_key", DEFAULT_LLAMACPP_API_KEY)
            .map_err(ConfigError::from)?
            .set_default("llamacpp.timeout_secs", DEFAULT_READ_TIMEOUT)
            .map_err(ConfigError::from)?
            .set_default("ollama.base_url", DEFAULT_OLLAMA_BASE_URL)
            .map_err(ConfigError::from)?
            .set_default("ollama.api_key", DEFAULT_OLLAMA_API_KEY)
            .map_err(ConfigError::from)?
            .set_default("ollama.timeout_secs", DEFAULT_READ_TIMEOUT)
            .map_err(ConfigError::from)?
            .set_default("system_prompt", DEFAULT_SYSTEM_PROMPT)
            .map_err(ConfigError::from)?)
    }

    /// Loads configuration from default values, values set in the configuration file env defined
    /// values and cli overrides.
    ///
    /// Overrides are applied in that order, with previous listed items being overridden by
    /// later ones.
    pub fn load(cli_options: Cli) -> Result<Self, AppError> {
        let mut config_builder = Self::build_default_config()?;

        if let Some(config_path) = find_config_file_path(cli_options.config.clone()) {
            config_builder = config_builder.add_source(config::File::from(config_path));
        }
        config_builder = config_builder.add_source(
            config::Environment::default()
                .prefix("ELELEMER")
                .prefix_separator("_")
                .separator("__"),
        );
        let settings = apply_cli_overrides(&cli_options, config_builder)?
            .build()
            .map_err(|err| AppError::Config(ConfigError::from(err)))?;

        let user_settings = settings
            .try_deserialize::<UserAppSettings>()
            .map_err(|err| AppError::Config(ConfigError::from(err)))?;

        let (runner, model, prompt, system_prompt) = match cli_options.command {
            Commands::Llamacpp(RunnerCommands::Run(run_args)) => {
                let api_config =
                    ApiConfig::try_from(user_settings.llamacpp).inspect_err(|_err| {
                        log::error!("Error initialising Llamacpp settings");
                    })?;

                (
                    RunnerSettings::Llamacpp(api_config),
                    run_args.model,
                    run_args.prompt,
                    user_settings.system_prompt,
                )
            }
            Commands::Ollama(RunnerCommands::Run(run_args)) => {
                let api_config = ApiConfig::try_from(user_settings.ollama).inspect_err(|_err| {
                    log::error!("Error initialising Ollama settings");
                })?;

                (
                    RunnerSettings::Ollama(api_config),
                    run_args.model,
                    run_args.prompt,
                    user_settings.system_prompt,
                )
            }
        };

        Ok(Self {
            runner,
            model,
            prompt,
            system_prompt,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use assert_fs::prelude::{FileWriteStr, PathChild};
    use clap_verbosity_flag::Verbosity;
    use miette::Diagnostic;
    use secrecy::SecretString;

    use crate::{
        cli::{Cli, Commands, RunArgs, RunnerCommands},
        configuration::{ApiConfig, AppSettings, UserApiConfig},
        test_helpers::LOGGING,
    };

    #[test]
    fn app_config_try_from_user_api_config_returns_error_for_invalid_api_key() {
        // assert
        let user_config = UserApiConfig {
            base_url: String::from("https://example.com"),
            api_key: SecretString::from("my\0secret"),
            timeout_secs: 12,
        };

        // act
        let outcome = ApiConfig::try_from(user_config).unwrap_err();

        // assert
        assert_eq!(&outcome.to_string(), "InvalidHeaderValue");
        let help = outcome
            .help()
            .map(|val| format!("{val}"))
            .expect("help method should be defined");
        assert_eq!(
            &help,
            "Check server API key value is correct; current value does not seem right."
        );
    }

    #[test]
    fn app_settings_load_converts_account_tilde_path_to_absolute_path() {
        // arrange
        let configuration_content = r#"system_prompt = "You are a helpful assistant"

[llamacpp]
base_url = "http://localhost:8080"
api_key = "not-required"
timeout_secs = 2400

[ollama]
base_url = "http://localhost:11434"
api_key = "not-required"
timeout_secs = 2400
"#;
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let _ = temp_dir
            .child("configuration.toml")
            .write_str(configuration_content);
        let config_path = temp_dir.join("configuration.toml");
        let cli = Cli {
            command: Commands::Ollama(RunnerCommands::Run(RunArgs {
                model: "gpt-oss:20b".to_owned(),
                prompt: "How far is the moon from planet Earth?".to_owned(),
                base_url: None,
                system_prompt: None,
                timeout: None,
            })),
            config: Some(config_path),

            #[cfg(feature = "internal-tools")]
            markdown_help: false,

            verbose: Verbosity::new(0, 0),
        };

        // act
        let result = AppSettings::load(cli).unwrap();

        // assert
        insta::assert_json_snapshot!(result);
    }

    #[test]
    fn app_settings_load_successfully_loads_llamacpp_config() {
        // arrange
        let cli = Cli {
            command: Commands::Llamacpp(RunnerCommands::Run(RunArgs {
                model: "gpt-oss:20b".to_owned(),
                prompt: "How many moons does Jupiter have?".to_owned(),
                base_url: None,
                system_prompt: None,
                timeout: None,
            })),
            config: None,

            #[cfg(feature = "internal-tools")]
            markdown_help: false,

            verbose: Verbosity::new(0, 0),
        };

        // act
        let result = AppSettings::load(cli).unwrap();

        // assert
        insta::assert_json_snapshot!(result);
    }

    #[tokio::test]
    async fn load_displays_error_detailing_llamacpp_config_is_issue_when_url_is_invalid() {
        // arrange
        let log_capture = LazyLock::force(&LOGGING);
        let command = Commands::Llamacpp(RunnerCommands::Run(RunArgs {
            model: String::from("qxz-supreme-sota:2t"),
            prompt: String::from("How many colours are there?"),
            base_url: Some(String::from("https//example.com")), // invalid URL
            timeout: Some(10),
            system_prompt: None,
        }));
        let cli = Cli {
            command,
            config: None,
            verbose: Verbosity::new(0, 0),
        };

        // act
        let outcome = AppSettings::load(cli).unwrap_err();

        // assert
        assert_eq!(
            outcome.to_string(),
            "Invalid URL: relative URL without a base"
        );
        let logs = log_capture.logs();
        assert!(&logs.contains("ERROR — Error initialising Llamacpp settings"));
    }

    #[tokio::test]
    async fn load_displays_error_detailing_ollama_config_is_issue_when_url_is_invalid() {
        // arrange
        let log_capture = LazyLock::force(&LOGGING);
        let command = Commands::Ollama(RunnerCommands::Run(RunArgs {
            model: String::from("qxz-supreme-sota:2t"),
            prompt: String::from("How many colours are there?"),
            base_url: Some(String::from("https")), // invalid URL
            timeout: Some(3),
            system_prompt: Some(String::from("Just try your best")),
        }));
        let cli = Cli {
            command,
            config: None,
            verbose: Verbosity::new(0, 0),
        };

        // act
        let outcome = AppSettings::load(cli).unwrap_err();

        // assert
        assert_eq!(
            outcome.to_string(),
            "Invalid URL: relative URL without a base"
        );
        let logs = log_capture.logs();
        assert!(&logs.contains("ERROR — Error initialising Ollama settings"));
    }
}
