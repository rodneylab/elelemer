use futures_util::io;

/// App configuration error
#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("{detail}")]
pub struct ConfigError {
    /// User-focused remedial suggestion
    #[help]
    pub advice: String,

    /// Error detail
    pub detail: String,
}

impl From<config::ConfigError> for ConfigError {
    fn from(value: config::ConfigError) -> Self {
        match value {
            config::ConfigError::FileParse { ref uri, .. } => match uri {
                Some(file_err) => Self {
                    advice: format!("Check syntax in `{file_err}`"),
                    detail: value.to_string(),
                },
                None => Self {
                    advice: "Check configuration syntax".to_owned(),
                    detail: value.to_string(),
                },
            },
            config::ConfigError::Foreign(ref err) => {
                if let Some(io_error) = err.downcast_ref::<std::io::Error>() {
                    if let io::ErrorKind::NotFound = io_error.kind() {
                        Self {
                            advice: "Config file not found; check it exists".to_owned(),
                            detail: io_error.to_string(),
                        }
                    } else {
                        Self {
                            advice: "Check file exists".to_owned(),
                            detail: "IO error while loading configuration file".to_owned(),
                        }
                    }
                } else {
                    Self {
                        advice: "Check file exists".to_owned(),
                        detail: "Unexpected error while loading configuration file".to_owned(),
                    }
                }
            }
            _ => {
                log::error!("Unexpected configuration error: {value:?}");
                Self {
                    advice: "Check file exists and syntax is valid".to_owned(),
                    detail: value.to_string(),
                }
            }
        }
    }
}

/// App input/output error
#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("Io error while {context}")]
pub struct IOError {
    /// User-focused remedial suggestion
    #[help]
    pub advice: String,

    /// Additional error context
    pub context: String,

    /// Error root cause
    pub cause: io::Error,
}

/// Runner API connection error
#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("Connection error while {context}")]
pub struct ConnectionError {
    /// User-focused remedial suggestion
    #[help]
    pub advice: String,

    /// Additional error context
    pub context: String,
}

/// Runner API response error
#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("Response error while {context}")]
pub struct ResponseError {
    /// User-focused remedial suggestion
    #[help]
    pub advice: String,

    /// Additional error context
    pub context: String,
}

/// Unexpected app error
///
/// App entered a state that was not considered possible enter
#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("Unexpected error while {context}")]
pub struct UnexpectedError {
    /// User-focused remedial suggestion
    #[help]
    pub advice: String,

    /// Additional error context
    pub context: String,

    /// Error root cause
    pub cause: String,
}

/// Runner API response stream error
#[derive(Debug, miette::Diagnostic, thiserror::Error)]
pub enum RunnerStreamError {
    /// Error connecting to runner API
    #[diagnostic(transparent)]
    #[error(transparent)]
    Connection(#[from] ConnectionError),

    /// Error while waiting for or parsing API response
    #[diagnostic(transparent)]
    #[error(transparent)]
    Response(#[from] ResponseError),

    /// Runner API error
    #[error(transparent)]
    Runner(#[from] eventsource_client::Error),

    /// Unexpected API runner error
    #[diagnostic(transparent)]
    #[error(transparent)]
    Unexpected(#[from] UnexpectedError),
}

/// Error initialising runner API SSE stream
#[derive(Debug, miette::Diagnostic, thiserror::Error)]
pub enum RunnerStreamInitialisationError {
    /// Unexpected runner API SSE stream initialisation error
    #[error(transparent)]
    Unexpected(#[from] UnexpectedError),
}

/// Global app error
#[derive(Debug, miette::Diagnostic, thiserror::Error)]
pub enum AppError {
    /// App configuration error
    #[diagnostic(transparent)]
    #[error(transparent)]
    Config(#[from] ConfigError),

    /// App input/output error
    #[diagnostic(transparent)]
    #[error(transparent)]
    IO(#[from] IOError),

    /// App runner API SSE streaming error
    #[diagnostic(transparent)]
    #[error(transparent)]
    RunnerStream(#[from] RunnerStreamError),

    /// App runner API SSE stream initialisation error
    #[diagnostic(transparent)]
    #[error(transparent)]
    RunnerStreamInitialisation(#[from] RunnerStreamInitialisationError),
}

#[cfg(test)]
mod tests {

    use miette::Diagnostic;

    use crate::errors::ConfigError;

    #[test]
    fn config_error_generates_expected_help_message() {
        // arrange
        let error = ConfigError {
            advice: String::from("Try rebooting your machine."),
            detail: String::from("Something went wrong."),
        };

        // act
        let help = error.help();

        // assert
        assert_eq!(&error.to_string(), "Something went wrong.");
        assert_eq!(&format!("{}", help.unwrap()), "Try rebooting your machine.");
    }

    #[test]
    fn config_error_from_file_parse_generates_expected_messages() {
        // arrange
        let error = config::Config::builder()
            .add_source(config::File::with_name("src/errors/fixtures/invalid.toml"))
            .build()
            .unwrap_err();

        // act
        let outcome = ConfigError::from(error);

        // assert
        let help = outcome.help();
        assert_eq!(
            &format!("{}", help.expect("Help should be Some")),
            "Check syntax in `src/errors/fixtures/invalid.toml`"
        );
        insta::assert_snapshot!(&outcome.to_string());
    }

    #[test]
    fn config_error_from_unexpected_error_generates_expected_messages() {
        // arrange
        let error = config::ConfigError::Message(String::from("Something went wrong!"));

        // act
        let outcome = ConfigError::from(error);

        // assert
        let help = outcome.help();
        assert_eq!(
            &format!("{}", help.expect("Help should be Some")),
            "Check file exists and syntax is valid"
        );
        insta::assert_snapshot!(&outcome.to_string());
    }

    #[test]
    fn config_error_from_foreign_error_generates_expected_messages() {
        // arrange
        let error = config::Config::builder()
            .add_source(config::File::with_name("fixtures/invalid.toml"))
            .build()
            .unwrap_err();

        // act
        let outcome = ConfigError::from(error);

        // assert
        let help = outcome.help();
        assert_eq!(
            &format!("{}", help.expect("Help should be Some")),
            "Config file not found; check it exists"
        );
        insta::assert_snapshot!(&outcome.to_string());
    }
}
