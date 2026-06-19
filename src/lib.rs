#![deny(missing_docs)]

//! # elelemer
//! `elelemer` implements logic for elelemer binary, which is a CLI utility for running local Large
//! Language Model completions using Llama.cpp and Ollama"

/// Cli argument parsing
pub mod cli;

/// Generate CLI markdown documentation
pub mod docs;

/// App setting defaults and setting generation from configuration file, env variables and cli
/// settings
pub mod configuration;

/// App error enums and structs
pub mod errors;

/// Http client and Sse stream
pub mod server_sent_events;

/// OpenAI-compatible API communication logic
pub mod openai_api;

/// Llamacpp and Ollama runner response parsing logic
pub mod runners;

/// Test helper logic including test logging
#[cfg(test)]
pub mod test_helpers;

/// App user interface logic
pub mod ui;
