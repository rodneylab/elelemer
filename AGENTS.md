# AGENTS.md

Rust CLI for local LLM completions via Llama.cpp and Ollama. Rust 1.93+,
edition 2024 (current stable edition). Current stable Rust version is 1.96.0.

## Commands

- `just` - list available tasks
- `just markdown-docs` - regenerate `docs/help.md` from CLI help
- `just insta-snapshot-review` - review snapshot changes
- `just insta-snapshot-clean` - delete unreferenced snapshots

## Testing

- `cargo test` - run unit tests (snapshot tests in `src/runners/`,
  `src/openai_api/`)
- `cargo insta review` - review snapshot diffs
- Uses `insta` for snapshot testing, `trycmd` for CLI tests (no CLI tests
  present yet)

## Linting & Formatting

- `cargo fmt` - uses `.rustfmt.toml` (Unix line endings, edition 2024)
- `cargo clippy` - strict lints defined in `[workspace.lints.clippy]`:
  - `allow_attributes`, `dbg_macro`, `print_stdout`, `get_unwrap`, `exit`
    are **denied**
  - `debug_assert_with_mut_call`, `iter_on_single_items`, `redundant_clone`
    are **warned**
- `dprint fmt` - formats JSON, Markdown, TOML (see `dprint.json`)
- `just comments` - find comments in Rust source (excludes `act`, `arrange`,
  `assert`)
- `just expects` - find `.expect()` and `.unwrap()` calls in Rust source

## Architecture

- `src/main.rs` - entrypoint, CLI parsing, dispatch to Ollama/llamacpp
- `src/cli.rs` - `clap`-based CLI definition
- `src/configuration.rs` - configuration handling (loads `configuration.toml`)
- `src/ui.rs` - streaming output display
- `src/runners/mod.rs` - runner trait and base implementation
- `src/runners/ollama/` - Ollama runner implementation
- `src/runners/llamacpp/` - llama.cpp runner implementation
- `src/openai_api/` - OpenAI-compatible API types (shared by both runners)

## CLI Structure

```console
elelemer ollama run <model> <prompt>
elelemer llamacpp run <model> <prompt>
```

## Configuration

- Default config: `configuration.toml` (localhost:8080 for llama.cpp,
  localhost:11434 for Ollama)
- Custom config: `elelemer --config <path>`

## Dependencies

- `clap` with derive for CLI parsing
- `tokio` async runtime
- `miette` for error handling
- `launchdarkly-sdk-transport` (Hyper transport for streamed events)
- `eventsource-client` for SSE streaming
