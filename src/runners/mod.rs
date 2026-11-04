/// Llamacpp Open AI-compatible API communication logic
pub mod llamacpp;

/// Ollama Open AI-compatible API communication logic
pub mod ollama;

mod serde_ext;

use std::{
    fmt,
    io::Write,
    time::{Duration, Instant},
};

use futures_util::{Stream, StreamExt, TryStreamExt};
use humantime::format_duration;
use indicatif::ProgressBar;

use crate::{
    configuration::ApiConfig,
    errors::{
        AppError, ConnectionError, IOError, ResponseError, RunnerStreamError, UnexpectedError,
    },
    openai_api::{FinalStreamChunk, OpenAIAPIClient, StreamChunk, Timings},
    ui::{METADATA_TEXT_STYLE, OutputWriter, REASONING_TEXT_STYLE, StdoutWriter},
};

/// Trait providing specialised chunk choice manipulation implementations for Llamacpp and Ollama runners
trait OpenAiCompatibleStreamChunkChoice {
    ///  OpenAI API-compatible response stream chunk finished marker
    fn finish_reason(&self) -> Option<&str>;
}

/// Trait providing specialised chunk object manipulation implementations for Llamacpp and Ollama runners
trait OpenAiCompatibleStreamChunkObject {
    /// Lllamacpp runner model name or alias
    fn model(&self) -> &str;

    /// Lllamacpp runner model token usage
    fn timings(&self) -> Option<Timings>;
}

fn extract_final_metadata<
    C: OpenAiCompatibleStreamChunkChoice,
    K: OpenAiCompatibleStreamChunkObject,
>(
    choice: C,
    chunk_object: K,
) -> (Option<String>, Option<Timings>) {
    if choice.finish_reason() == Some("stop") {
        (
            Some(chunk_object.model().to_owned()),
            chunk_object.timings(),
        )
    } else {
        (None, None)
    }
}

fn extract_final_metadata_usage<K: OpenAiCompatibleStreamChunkObject>(
    chunk_object: K,
) -> (Option<String>, Option<Timings>) {
    let model = Some(chunk_object.model().to_owned());
    let timings = chunk_object.timings();

    (model, timings)
}

/// Map 'error' encountered while parsing runner response stream chunk.
///
/// Parsing logic assumes chunks are JSON, though the final chunk will be a string message.  This
/// function converts that message into a chunk.
///
/// # Returns
/// - `None` for non-JSON chunks (error can be handled by caller); and
/// - `Some[StreamChunk])` containing the "[DONE]" message for valid termination messages.
pub fn map_stream_chunk_json_parse_error(
    chunk: &str,
    name: &str,
    err: serde_json::Error,
) -> Option<StreamChunk> {
    // "[DONE]" message is not valid JSON, like other messages, so will generate an error when
    // encountered by the JSON parser
    if chunk == "[DONE]" {
        Some(StreamChunk::Final(FinalStreamChunk {
            model: None,
            timings: None,
        }))
    } else {
        tracing::error!("Unable to deserialise {name} stream chunk `{chunk}`: {err:?}",);

        None
    }
}

/// OpenAI API-compatible response stream chunk message
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct OpenAiCompatibleStreamChunkChoiceDeltaContent {
    /// Message body
    pub content: String,
}

/// OpenAI API-compatible response stream chunk reasoning message
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct OpenAiCompatibleStreamChunkChoiceDeltaReasoningContent {
    /// Message body
    pub reasoning: String,
}

/// Helper function used in From `LlamacppStreamChunk` and `OllamaStreamChunk` for
/// [`StreamChunk`] implementation
pub fn parse_open_ai_compatible_stream_chunk_string(chunk: String) -> StreamChunk {
    if chunk == "DONE" {
        StreamChunk::Final(FinalStreamChunk {
            model: None,
            timings: None,
        })
    } else {
        StreamChunk::Empty
    }
}

/// Runner stream response chunk
#[derive(Debug, serde::Deserialize)]
pub struct RunnerResponse {
    /// Content body
    pub content: String,

    /// Runner model
    pub model: String,

    /// Runner model response tokens
    pub tokens_predicted: u32,

    /// Runner model prompt tokens
    pub tokens_evaluated: u32,
}

impl<W: Write> From<StreamConsumer<'_, W>> for RunnerResponse {
    fn from(value: StreamConsumer<W>) -> Self {
        Self {
            content: value.displayer.complete_content,
            model: value.model.unwrap_or("Unknown model".to_owned()),
            tokens_predicted: value.tokens_predicted.unwrap_or_default(),
            tokens_evaluated: value.tokens_evaluated.unwrap_or_default(),
        }
    }
}

fn update_stream_metadata(chunk: StreamChunk) -> (Option<String>, Option<u32>, Option<u32>) {
    debug_assert!(matches!(chunk, StreamChunk::Final(_)));

    if let StreamChunk::Final(FinalStreamChunk { model, ref timings }) = chunk {
        (
            model,
            timings.as_ref().map(|val| val.prompt_n),
            timings.as_ref().map(|val| val.predicted_n),
        )
    } else {
        (None, None, None)
    }
}

/// Struct for displaying response stream chunks to user interface for immediate display and pushing
/// chunks into a buffer, for potential use elsewhere.
struct ResponseStreamDisplayer<'a, W: Write> {
    reasoning: bool,

    /// Complete response so far
    pub complete_content: String,

    writer: StdoutWriter<'a, W>,
}

impl<'a, W: Write> ResponseStreamDisplayer<'a, W> {
    /// Create a ['ResponseStreamDisplayer']
    pub fn new(stdout_handle: &'a mut W) -> Self {
        Self {
            reasoning: false,
            complete_content: String::new(),
            writer: StdoutWriter::new(stdout_handle),
        }
    }

    /// Display a stream chunk to a buffer.
    ///
    /// Currently just used to display to stdout.
    fn display_stream_chunk(&mut self, chunk: &StreamChunk) -> Result<(), IOError> {
        match chunk {
            StreamChunk::Content(content) => {
                if self.reasoning {
                    self.reasoning = false;
                    let heading = "\n\n...reasoning completed.\n\n\n";
                    self.writer
                        .write_chunk(heading, Some(REASONING_TEXT_STYLE))?;
                    self.complete_content.push_str(heading);
                }
                self.writer.write_chunk(content, None)?;
                self.writer.flush()?;
                self.complete_content.push_str(content);
            }
            StreamChunk::Reasoning(content) => {
                if !self.reasoning {
                    self.reasoning = true;
                    let heading = "\nReasoning...\n\n";
                    self.writer
                        .write_chunk(heading, Some(REASONING_TEXT_STYLE))?;
                    self.complete_content.push_str(heading);
                }
                self.writer
                    .write_chunk(content, Some(REASONING_TEXT_STYLE))?;
                self.writer.flush()?;
                self.complete_content.push_str(content);
            }
            StreamChunk::Empty => {}
            StreamChunk::Final(_final_stream_chunk) => {}
        }

        Ok(())
    }
}

fn format_final_metadata(elapsed: &Duration, tokens_predicted: Option<u32>) -> String {
    let duration_milliseconds = elapsed.as_millis();
    let duration = if duration_milliseconds > 1_000 {
        format_duration(Duration::from_secs(elapsed.as_secs()))
    } else {
        format_duration(Duration::from_millis(
            u64::try_from(duration_milliseconds)
                .expect("u128 value should be under 1000, and fit into a u64"),
        ))
    };

    if let Some(tokens_predicted) = tokens_predicted
        && duration_milliseconds > 0
    {
        let tokens_per_second =
            (1_000.0 * f64::from(tokens_predicted) / elapsed.as_secs_f64()).round() / 1_000.0;

        format!("\n{duration}, {tokens_per_second}t/s\n")
    } else {
        format!("\n{duration}\n")
    }
}

enum StreamState {
    Idle,
    FirstChunkReceived,
    FinalChunkReceived,
}

struct StreamFinalMetadata<'a> {
    duration: &'a Duration,
    tokens_predicted: Option<u32>,
}

impl<'a> fmt::Display for StreamFinalMetadata<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            format_final_metadata(self.duration, self.tokens_predicted)
        )
    }
}

impl<'a> StreamFinalMetadata<'a> {
    fn new(duration: &'a Duration, tokens_predicted: Option<u32>) -> StreamFinalMetadata<'a> {
        Self {
            duration,
            tokens_predicted,
        }
    }
}

struct StreamConsumer<'a, W: Write> {
    displayer: ResponseStreamDisplayer<'a, W>,
    state: StreamState,
    spinner: Option<ProgressBar>,
    model: Option<String>,
    tokens_evaluated: Option<u32>,
    tokens_predicted: Option<u32>,
    start: Instant,
}

impl<'a, W: Write> StreamConsumer<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        let displayer = ResponseStreamDisplayer::new(writer);
        let spinner = displayer.writer.show_spinner();

        Self {
            displayer,
            state: StreamState::Idle,
            spinner,
            model: None,
            tokens_evaluated: None,
            tokens_predicted: None,
            start: Instant::now(),
        }
    }

    fn stop_spinner(&mut self) -> bool {
        if let Some(spinner) = &self.spinner {
            spinner.finish();
            self.spinner = None;

            true
        } else {
            false
        }
    }

    fn handle_received_first_chunk(&mut self) {
        self.state = StreamState::FirstChunkReceived;
        self.stop_spinner();
    }

    fn consume_chunk(&mut self, chunk: StreamChunk) -> Result<(), AppError>
    where
        W: Write,
    {
        if let StreamState::Idle = self.state {
            self.handle_received_first_chunk();
        }

        self.displayer.display_stream_chunk(&chunk)?;

        if let StreamChunk::Final(_) = chunk {
            (self.model, self.tokens_evaluated, self.tokens_predicted) =
                update_stream_metadata(chunk);
            self.state = StreamState::FinalChunkReceived;
        }

        Ok(())
    }

    fn finalise(&mut self) -> Result<(), AppError> {
        let statistics =
            StreamFinalMetadata::new(&self.start.elapsed(), self.tokens_predicted).to_string();
        self.displayer
            .writer
            .write_chunk(&statistics, Some(METADATA_TEXT_STYLE))?;
        self.displayer.writer.flush()?;

        Ok(())
    }
}

async fn process_response_stream<W: Write>(
    writer: &mut W,
    stream: impl Stream<Item = Result<StreamChunk, RunnerStreamError>>,
) -> Result<RunnerResponse, AppError> {
    futures_util::pin_mut!(stream);

    let mut stream_consumer = StreamConsumer::new(writer);

    loop {
        match stream.try_next().await {
            Ok(chunk_option) => {
                if let Some(chunk) = chunk_option {
                    stream_consumer.consume_chunk(chunk)?;

                    if let StreamState::FinalChunkReceived = stream_consumer.state {
                        break;
                    }
                }
            }
            Err(err) => {
                return Err(err.into());
            }
        }
    }

    stream_consumer.finalise()?;

    Ok(stream_consumer.into())
}

fn map_event_source_error(err: eventsource_client::Error, name: &str) -> RunnerStreamError {
    match err {
        eventsource_client::Error::Eof => {
            tracing::debug!(
                "Received stream EOF from {name} server (HTTP response stream has ended.)"
            );

            RunnerStreamError::from(err)
        }
        eventsource_client::Error::Transport(transport_error) => {
            tracing::error!("{transport_error:?}");

            let advice = format!(
                "Check the {name} server is running or try increasing read timeout by passing \
                        `--timeout 600`, for a 600-second timeout, for example.",
            );

            RunnerStreamError::from(ConnectionError {
                advice,
                context: format!("sending request to {name} server"),
            })
        }
        eventsource_client::Error::UnexpectedResponse(ref response, ref error_body) => {
            if response.status() == 404 {
                return RunnerStreamError::from(ResponseError {
                    advice: format!("Check the requested model is available on the {name} server",),
                    context: format!("sending request to {name} server"),
                });
            }
            tracing::error!(
                "Received unexpected response from {name} server: {{response: {response:?},
                            error_body: {error_body:?}}}",
            );
            RunnerStreamError::from(err)
        }
        _ => RunnerStreamError::from(UnexpectedError {
            advice: "contact support".to_owned(),
            context: format!("sending request to {name} server"),
            cause: format!("Error streaming {name} response: {err:?}"),
        }),
    }
}

/// LLM Runner core logic.
///
/// Acts as base.  LLamacpp and Ollama have implement parsers used in this runner implementation.
pub struct Runner<'a, P: LlmRunnerParser> {
    client: OpenAIAPIClient<'a>,
    parser: P,
}

impl<'a, P> Runner<'a, P>
where
    P: LlmRunnerParser,
{
    fn parse_events(
        &self,
        client: impl eventsource_client::Client,
    ) -> impl Stream<Item = Result<Option<StreamChunk>, RunnerStreamError>> {
        client
            .stream()
            .map_ok(|event| match event {
                eventsource_client::SSE::Event(ref ev) => self.parser.parse_stream_chunk(&ev.data),
                eventsource_client::SSE::Connected(connection_details) => {
                    tracing::info!("Connected to {} server stream", self.parser.name());
                    tracing::trace!(
                        "{} server connection details: {connection_details:?}",
                        self.parser.name()
                    );

                    None
                }
                eventsource_client::SSE::Comment(comment) => {
                    tracing::debug!("Received {} stream comment: {comment}", self.parser.name());

                    None
                }
            })
            .map_err(|err| map_event_source_error(err, self.parser.name()))
    }

    async fn get_response_stream(
        &self,
        events_client: impl eventsource_client::Client,
    ) -> Result<impl Stream<Item = Result<StreamChunk, RunnerStreamError>>, RunnerStreamError> {
        Ok(self.parse_events(events_client).filter_map(|val| async {
            match val {
                Ok(value) => value.map(Ok),
                Err(err) => Some(Err(err)),
            }
        }))
    }

    async fn run(
        &self,
        writer: &mut impl Write,
        model: &str,
        system_prompt: &str,
        prompt: &str,
    ) -> Result<(), AppError> {
        tracing::info!(
            "Sending prompt to {} server and awaiting response...",
            self.parser.name()
        );
        tracing::debug!("Prompt: {prompt}");

        let client = self.get_openai_api_client();
        let events_client = client.create_server_events_client(model, system_prompt, prompt)?;
        let stream = self.get_response_stream(events_client).await?;
        futures_util::pin_mut!(stream);

        let response = process_response_stream(writer, stream).await?;
        tracing::debug!("{response:?}");

        Ok(())
    }

    /// Invoke the runner
    pub async fn invoke(
        &self,
        writer: &mut impl Write,
        system_prompt: &str,
        model: &str,
        prompt: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Using {} runner", self.parser.name());
        self.run(writer, model, system_prompt, prompt).await?;

        Ok(())
    }

    fn get_openai_api_client(&self) -> &OpenAIAPIClient<'_> {
        &self.client
    }

    /// Create a new runner
    pub fn new(parser: P, api_config: &'a ApiConfig) -> Result<Self, AppError> {
        Ok(Self {
            client: OpenAIAPIClient::new(api_config)?,
            parser,
        })
    }
}

/// Trait providing specialised parser implementations for Llamacpp and Ollama runners
pub trait LlmRunnerParser {
    /// Function for parsing a JSON-encoded OpenAI API-compatible response stream chunk into
    /// a [`StreamChunk`]
    fn parse_stream_chunk(&self, chunk: &str) -> Option<StreamChunk>;

    /// Human-readable runner name for logging and UI messages
    fn name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::runners::format_final_metadata;

    #[test]
    fn format_final_metadata_returns_expected_output() {
        // arrange
        let duration = Duration::new(42, 456_000_000);

        // act
        let outcome = format_final_metadata(&duration, Some(567));

        // assert
        assert_eq!(&outcome, "\n42s, 13.355t/s\n");
    }

    #[test]
    fn format_final_metadata_returns_expected_output_for_long_elapsed_time() {
        // arrange
        let duration = Duration::new(3_600 * 30 + 60 * 3 + 24, 456_000_000);

        // act
        let outcome = format_final_metadata(&duration, Some(9_876));

        // assert
        assert_eq!(&outcome, "\n1day 6h 3m 24s, 0.091t/s\n");
    }

    #[test]
    fn format_final_metadata_returns_expected_output_for_small_duration() {
        // arrange
        let duration = Duration::new(0, 789);

        // act
        let outcome = format_final_metadata(&duration, Some(567));

        // assert
        assert_eq!(&outcome, "\n0s\n");
    }

    #[test]
    fn format_final_metadata_returns_expected_output_for_missing_token_usage() {
        // arrange
        let duration = Duration::new(42, 456_000_000);

        // act
        let outcome = format_final_metadata(&duration, None);

        // assert
        assert_eq!(&outcome, "\n42s\n");
    }
}
