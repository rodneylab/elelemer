use std::path::Path;

use clap::Parser;

use crate::errors::IOError;

/// Write the CLI documentation to a file
pub fn write_markdown_docs_to_file<P: AsRef<Path>, R: Parser>(path: P) -> Result<(), IOError> {
    let markdown = clap_markdown::help_markdown::<R>();

    std::fs::write(&path, markdown).map_err(|err| IOError {
        advice: format!(
            "Check you have write access to markdown documentation path `{}`",
            path.as_ref().display()
        ),
        context: "writing markdown help to file".to_owned(),
        cause: err,
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use miette::Diagnostic;

    use crate::{cli::Cli, docs::write_markdown_docs_to_file};

    #[test]
    fn write_markdown_docs_to_file_generates_expected_file() {
        // arrange
        let temp_dir =
            assert_fs::TempDir::new().expect("Should be able to create a temporary directory");
        let path = temp_dir.join("docs.md");

        // act
        let outcome = write_markdown_docs_to_file::<_, Cli>(&path);

        // assert
        assert!(outcome.is_ok());
        assert!(path.is_file());
        let content =
            read_to_string(path).expect("Should successfully read file into string buffer");
        insta::assert_snapshot!(content);
    }

    #[test]
    fn write_markdown_docs_to_file_handles_invalid_path() {
        // arrange
        let path = std::env::current_dir()
            .expect("Current directory should exist and user should have read permissions")
            .join("does-not-exist")
            .join("docs.md");

        // act
        let outcome = write_markdown_docs_to_file::<_, Cli>(&path).unwrap_err();

        // assert
        assert_eq!(
            &outcome.to_string(),
            "Io error while writing markdown help to file"
        );

        let help = outcome
            .help()
            .map(|val| format!("{val}"))
            .expect("Error should have help defined");
        assert!(help.contains("Check you have write access to markdown documentation path `"));
        assert!(help.contains("does-not-exist"));
        assert!(help.contains("docs.md"));
    }
}
