use std::{
    io::{self, Read, Write},
    sync::LazyLock,
};

use env_logger::WriteStyle;
use log::LevelFilter;
use tempfile::NamedTempFile;

/// Test logger
pub static LOGGING: LazyLock<LogCapture> = LazyLock::new(LogCapture::default);

/// Test log capture
pub struct LogCapture {
    temp_file: NamedTempFile,
}

impl Default for LogCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl LogCapture {
    fn new() -> Self {
        let temp_file = NamedTempFile::new()
            .expect("Should have sufficient permissions to create a temporary file");
        env_logger::Builder::new()
            .write_style(WriteStyle::Always)
            .filter(None, LevelFilter::Info)
            .format(|buf, record| writeln!(buf, "{} — {}", record.level(), record.args()))
            .target(env_logger::Target::Pipe(Box::new(io::BufWriter::new(
                temp_file
                    .reopen()
                    .expect("Should be able to open temporary file"),
            ))))
            .init();

        LogCapture { temp_file }
    }

    /// Extract captured logs from logger
    pub fn logs(&self) -> String {
        let mut result = String::new();
        self.temp_file
            .reopen()
            .expect("Should be able to reopen temporary file")
            .read_to_string(&mut result)
            .expect("Should be able to read temporary file");

        result
    }
}
