use std::{
    env,
    path::{Path, PathBuf},
};

const CONFIGURATION_FILE_NAME: &str = "configuration.toml";

fn get_configuration_path<P: AsRef<Path>>(dir_path: P) -> Option<PathBuf> {
    let config_path = dir_path.as_ref().join(CONFIGURATION_FILE_NAME);
    if config_path.is_file() {
        log::info!("Using configuration file {}", config_path.display());

        Some(config_path)
    } else {
        log::debug!(
            "Unable to find configuration file at {}",
            config_path.display()
        );

        None
    }
}

fn get_current_dir_configuration_path() -> Option<PathBuf> {
    let Ok(current_dir) = env::current_dir() else {
        log::warn!("Could not determine current working directory");

        return None;
    };

    get_configuration_path(current_dir)
}

fn get_config_path_from_dir<P: AsRef<Path>>(dir: P) -> Option<PathBuf> {
    let config_path = dir.as_ref().join(CONFIGURATION_FILE_NAME);
    if config_path.is_file() {
        log::info!("Using configuration file {}", config_path.display());

        Some(config_path)
    } else {
        log::debug!(
            "Unable to find configuration file at {}",
            config_path.display()
        );

        None
    }
}

fn get_config_dir_configuration_path() -> Option<PathBuf> {
    let Some(config_dir) = dirs::config_local_dir() else {
        log::warn!("Could not determine local configuration directory");

        return None;
    };

    get_config_path_from_dir(config_dir.join(env!("CARGO_PKG_NAME")))
}

pub fn find_config_file_path(file_path: Option<PathBuf>) -> Option<PathBuf> {
    file_path
        .or_else(get_current_dir_configuration_path)
        .or_else(get_config_dir_configuration_path)
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::LazyLock};

    use assert_fs::fixture::{FileWriteStr as _, PathChild};

    use crate::{
        configuration::config_path::{
            get_config_path_from_dir, get_current_dir_configuration_path,
        },
        test_helpers::LOGGING,
    };

    #[cfg(target_family = "unix")]
    use crate::configuration::config_path::get_configuration_path;

    #[test]
    #[cfg(target_family = "unix")]
    fn get_configuration_path_returns_none_when_local_config_file_does_not_exist() {
        // arrange
        let dir = PathBuf::from("does-not-exist");

        // act
        let outcome = get_configuration_path(dir);

        // assert
        assert!(outcome.is_none());
    }

    #[test]
    fn get_current_dir_configuration_path_returns_expected_result() {
        // arrange

        // act
        let outcome = get_current_dir_configuration_path();

        // assert
        let outcome_path = outcome.expect("Expected path to be some");
        assert!(outcome_path.ends_with("elelemer/configuration.toml"));
    }

    #[test]
    fn get_current_dir_configuration_path_handles_missing_file() {
        // arrange

        // act
        let outcome = get_current_dir_configuration_path();

        // assert
        let outcome_path = outcome.expect("Expected path to be some");
        assert!(outcome_path.ends_with("elelemer/configuration.toml"));
    }

    #[test]
    fn get_config_path_from_dir_returns_path() {
        // arrange
        let log_capture = LazyLock::force(&LOGGING);
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let config_path = temp_dir.join("configuration.toml");
        let _ = temp_dir.child("configuration.toml").write_str("\n\n");

        // act
        let outcome = get_config_path_from_dir(temp_dir);

        // assert
        let outcome_path = outcome.expect("Expected path to be some");
        assert_eq!(outcome_path, config_path);
        let logs = log_capture.logs();
        assert!(logs.contains(&format!(
            "INFO — Using configuration file {}",
            config_path.display()
        )));
    }
}
