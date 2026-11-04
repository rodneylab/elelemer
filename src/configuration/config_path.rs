use std::{
    env::{self},
    path::{Path, PathBuf},
};

use crate::errors::ConfigError;

const CONFIGURATION_FILE_NAME: &str = "configuration.toml";

fn verify_config_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, ConfigError> {
    let path = path.as_ref();

    if path.is_file() {
        Ok(path.to_path_buf())
    } else {
        Err(ConfigError {
            advice: format!(
                "Check configuration file {} is a file with read permissions",
                path.display()
            ),
            detail: String::from("verifying configuration file path"),
        })
    }
}

fn get_current_dir_configuration_path() -> Result<PathBuf, ConfigError> {
    let Ok(current_dir) = env::current_dir() else {
        return Err(ConfigError {
            advice: String::from(
                "Make sure you have read permissions for the current directory, if you are \
                expecting the app to use a configuration file from it.",
            ),
            detail: String::from(
                "Could not determine current working directory while searching for configuration",
            ),
        });
    };

    verify_config_path(current_dir.join(CONFIGURATION_FILE_NAME))
}

fn get_config_dir_configuration_path() -> Result<PathBuf, ConfigError> {
    let Some(config_dir) = dirs::config_local_dir() else {
        return Err(ConfigError {
            advice: String::from(
                "Alternatively, try passing a configuration path in app run command, if you need \
                to use a configuration file.",
            ),
            detail: String::from("Could not determine local configuration directory"),
        });
    };

    let path = config_dir
        .join(env!("CARGO_PKG_NAME"))
        .join(CONFIGURATION_FILE_NAME);

    verify_config_path(path)
}

/// Searches for a configuration file in the current directory first, falling back to system
/// configuration directory.
pub fn find_configuration_path(file_path: Option<PathBuf>) -> Result<Option<PathBuf>, ConfigError> {
    if let Some(path) = file_path {
        verify_config_path(&path)?;

        return Ok(Some(path));
    }

    match get_current_dir_configuration_path() {
        Ok(path) => {
            return Ok(Some(path));
        }
        Err(err) => {
            tracing::warn!("{err}");
        }
    }

    match get_config_dir_configuration_path() {
        Ok(path) => Ok(Some(path)),
        Err(err) => {
            tracing::warn!("{err}");

            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use assert_fs::fixture::{FileTouch, PathChild};
    use tracing_test::traced_test;

    use crate::configuration::config_path::{
        CONFIGURATION_FILE_NAME, find_configuration_path, get_current_dir_configuration_path,
        verify_config_path,
    };

    #[test]
    fn verify_config_path_returns_error_when_local_config_file_does_not_exist() {
        // arrange
        let dir = PathBuf::from("does-not-exist");

        // act
        let outcome = verify_config_path(dir).unwrap_err();

        // assert
        assert_eq!(&outcome.to_string(), "verifying configuration file path");
    }

    #[test]
    fn get_current_dir_configuration_path_returns_expected_result() {
        // arrange

        // act
        let outcome = get_current_dir_configuration_path();

        // assert
        let outcome_path = outcome.expect("Expected path to be some");
        assert!(outcome_path.ends_with(CONFIGURATION_FILE_NAME));

        // assuming package name matches test run directory (as current_dir)
        let config_dir_name = env!("CARGO_PKG_NAME");
        assert!(outcome_path.parent().unwrap().ends_with(config_dir_name));
    }

    #[test]
    #[traced_test]
    fn find_config_file_path_returns_path() {
        // arrange
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let config_path = temp_dir.join(CONFIGURATION_FILE_NAME);
        temp_dir.child(CONFIGURATION_FILE_NAME).touch().unwrap();

        // act
        let outcome = find_configuration_path(Some(config_path.clone()));

        // assert
        let outcome_path = outcome
            .expect("Expected outcome to be Ok")
            .expect("Expected path to be some");
        assert_eq!(outcome_path, config_path);
    }
}
