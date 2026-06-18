use std::{io::Write, time::Duration};

use indicatif::ProgressBar;
use owo_colors::{OwoColorize, Style};

use crate::errors::IOError;

/// Text style for LLM reasoning-mode response output
pub const REASONING_TEXT_STYLE: Style = Style::new().blue();

/// Default stdout text style
pub const DEFAULT_TEXT_STYLE: Style = Style::new().default_color();

/// Map std::io::Error to [`IOError`]
pub fn map_stdout_io_error(err: std::io::Error) -> IOError {
    IOError {
        advice: "Contact support".to_owned(),
        context: "Displaying runner response to stdout".to_owned(),
        cause: err,
    }
}

/// Logic for outputting LLM responses to stdout.
///
/// Written as a trait to facilitate testing and wider used.
pub trait OutputWriter {
    /// Write chunk to an output sink.
    fn write_chunk(&mut self, chunk: &str, style: Option<Style>) -> Result<(), IOError>;

    /// Flush the output sink
    fn flush(&mut self) -> Result<(), IOError>;

    /// Show a spinner in the output.
    fn show_spinner(&self) -> Option<ProgressBar> {
        let bar = ProgressBar::new_spinner();
        bar.enable_steady_tick(Duration::from_millis(100));

        Some(bar)
    }
}

/// Struct for outputting LLM output to stdout
pub struct StdoutWriter<'a, W: Write> {
    stdout_handle: &'a mut W,
}

impl<W: Write> OutputWriter for StdoutWriter<'_, W> {
    fn write_chunk(&mut self, chunk: &str, style: Option<Style>) -> Result<(), IOError> {
        write!(
            self.stdout_handle,
            "{}",
            chunk.style(style.unwrap_or(DEFAULT_TEXT_STYLE))
        )
        .map_err(map_stdout_io_error)
    }

    fn flush(&mut self) -> Result<(), IOError> {
        self.stdout_handle.flush().map_err(map_stdout_io_error)
    }
}

impl<'a, W: Write> StdoutWriter<'a, W> {
    /// Create a new [`StdoutWriter`]
    pub fn new(writer: &'a mut W) -> Self {
        Self {
            stdout_handle: writer,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ui::{OutputWriter, REASONING_TEXT_STYLE, StdoutWriter, map_stdout_io_error};

    #[test]
    fn handle_std_out_io_error_generates_expected_value() {
        // arrange
        let error = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Something went wrong!");

        // act
        let outcome = map_stdout_io_error(error);

        // assert
        insta::assert_snapshot!(format!("{outcome:?}"));
    }

    #[test]
    fn output_writer_write_chunk_generates_expected_output() {
        // arrange
        let mut buffer = Vec::new();
        let mut writer = StdoutWriter::new(&mut buffer);
        let chunk = "Good day to you!";

        // act
        writer.write_chunk(chunk, None).unwrap();

        // assert
        insta::assert_snapshot!(String::from_utf8_lossy(&buffer));
    }

    #[test]
    fn output_writer_write_chunk_generates_expected_output_with_formatting() {
        // arrange
        let mut buffer = Vec::new();
        let mut writer = StdoutWriter::new(&mut buffer);
        let chunk = "Hmmm... 🤔";

        // act
        writer
            .write_chunk(chunk, Some(REASONING_TEXT_STYLE))
            .unwrap();

        // assert
        insta::assert_snapshot!(String::from_utf8_lossy(&buffer));
    }
}
