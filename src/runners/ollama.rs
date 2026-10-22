use crate::{
    openai_api::{FinalStreamChunk, StreamChunk, Timings},
    runners::{
        LlmRunnerParser, OpenAiCompatibleStreamChunkChoice,
        OpenAiCompatibleStreamChunkChoiceDeltaContent,
        OpenAiCompatibleStreamChunkChoiceDeltaReasoningContent, OpenAiCompatibleStreamChunkObject,
        extract_final_metadata, map_stream_chunk_json_parse_error,
        parse_open_ai_compatible_stream_chunk_string,
        serde_ext::deserialise_ollama_open_api_helper,
    },
};

// See
// <https://developers.openai.com/api/reference/resources/chat/subresources/completions/streaming-events>
// for OpenAI documentation

/// OpenAI API-compatible response stream chunk choice delta
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(untagged)]
pub enum OllamaStreamChunkChoiceDelta {
    /// OpenAI API-compatible response stream chunk message
    Content(OpenAiCompatibleStreamChunkChoiceDeltaContent),

    /// OpenAI API-compatible response stream chunk reasoning message
    ReasoningContent(OpenAiCompatibleStreamChunkChoiceDeltaReasoningContent),
}

/// OpenAI API-compatible response stream chunk choice
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct OllamaStreamChunkChoice {
    ///  OpenAI API-compatible response stream chunk finished marker
    pub finish_reason: Option<String>,

    ///  OpenAI API-compatible response stream chunk delta
    #[serde(default, deserialize_with = "deserialise_ollama_open_api_helper")]
    pub delta: Option<OllamaStreamChunkChoiceDelta>,
}

impl OpenAiCompatibleStreamChunkChoice for &OllamaStreamChunkChoice {
    fn finish_reason(&self) -> Option<&str> {
        self.finish_reason.as_deref()
    }
}

/// Lllamacpp runner model token usage
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct OllamaStreamChunkUsage {
    /// Number of tokens in the runner model prompt
    pub prompt_tokens: u32,

    /// Number of tokens in the runner model generated completion
    pub completion_tokens: u32,
}

impl From<&OllamaStreamChunkUsage> for Timings {
    fn from(value: &OllamaStreamChunkUsage) -> Self {
        Self {
            prompt_n: value.prompt_tokens,
            predicted_n: value.completion_tokens,
        }
    }
}

/// OpenAI API-compatible response stream chunk object
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct OllamaStreamChunkObject {
    ///  OpenAI API-compatible response stream chunk choices
    pub choices: Vec<OllamaStreamChunkChoice>,

    /// Runner model name or alias
    pub model: String,

    #[cfg(test)]
    system_fingerprint: String,

    #[cfg(test)]
    object: String,

    usage: Option<OllamaStreamChunkUsage>,
}

impl OpenAiCompatibleStreamChunkObject for &OllamaStreamChunkObject {
    fn model(&self) -> &str {
        &self.model
    }

    fn timings(&self) -> Option<Timings> {
        self.usage.as_ref().map(Timings::from)
    }
}

///  OpenAI API-compatible response stream chunk
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(untagged)]
pub enum OpenAiApiCompatibleStreamChunk {
    ///  OpenAI API-compatible response stream chunk string
    String(String),

    ///  OpenAI API-compatible response stream chunk object
    Object(OllamaStreamChunkObject),
}

///  OpenAI API-compatible response stream chunk
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(untagged)]
pub enum OllamaStreamChunk {
    ///  OpenAI API-compatible response stream chunk string
    String(String),

    ///  OpenAI API-compatible response stream chunk object
    Object(OllamaStreamChunkObject),
}

impl From<OllamaStreamChunk> for StreamChunk {
    fn from(value: OllamaStreamChunk) -> Self {
        match value {
            OllamaStreamChunk::String(chunk) => parse_open_ai_compatible_stream_chunk_string(chunk),
            OllamaStreamChunk::Object(chunk_object) => {
                if let Some(choice) = chunk_object.choices.first() {
                    match &choice.delta {
                        None => {
                            let (model, timings) = extract_final_metadata(choice, &chunk_object);

                            Self::Final(FinalStreamChunk { model, timings })
                        }
                        Some(OllamaStreamChunkChoiceDelta::Content(value)) => {
                            Self::Content(value.content.clone())
                        }
                        Some(OllamaStreamChunkChoiceDelta::ReasoningContent(value)) => {
                            Self::Reasoning(value.reasoning.clone())
                        }
                    }
                } else {
                    Self::Empty
                }
            }
        }
    }
}

/// Ollama runner
#[derive(Default)]
pub struct OllamaRunner;

impl LlmRunnerParser for OllamaRunner {
    fn parse_stream_chunk(&self, chunk: &str) -> Option<StreamChunk> {
        log::debug!("Ollama stream chunk: {chunk}");
        let chunk: Option<OllamaStreamChunk> = match serde_json::from_str(chunk) {
            Ok(value) => {
                log::debug!("Deserialised {} stream chunk: `{chunk}`", self.name());

                Some(value)
            }
            Err(err) => {
                return map_stream_chunk_json_parse_error(chunk, self.name(), err);
            }
        };

        chunk.map(StreamChunk::from)
    }

    fn name(&self) -> &'static str {
        "ollama"
    }
}

impl OllamaRunner {
    /// Create a ['OllamaRunner']
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use crate::runners::ollama::OllamaStreamChunkObject;

    #[test]
    fn ollama_stream_chunk_deserialises_reasoning_stream_message() {
        // arrange
        let message_json_str = r#"
{
  "id": "chatcmpl-998",
  "object": "chat.completion.chunk",
  "created": 1779462631,
  "model": "qwen3-vl:8b",
  "system_fingerprint": "fp_ollama",
  "choices": [
    {
      "index": 0,
      "delta": { "role": "assistant", "content": "", "reasoning": " checking" },
      "finish_reason": null
    }
  ]
}"#;

        // act
        let chunk: OllamaStreamChunkObject = serde_json::from_str(message_json_str).unwrap();

        // assert
        insta::assert_json_snapshot!(chunk);
    }

    #[test]
    fn ollama_stream_chunk_deserialises_content_stream_message() {
        // arrange
        let message_json_str = r#"
{
  "id": "chatcmpl-658",
  "object": "chat.completion.chunk",
  "created": 1779465659,
  "model": "phi4-mini:3.8b",
  "system_fingerprint": "fp_ollama",
  "choices": [
    { "index": 0, "delta": { "role": "assistant", "content": " task" }, "finish_reason": null }
  ]
}"#;

        // act
        let chunk: OllamaStreamChunkObject = serde_json::from_str(message_json_str).unwrap();

        // assert
        insta::assert_json_snapshot!(chunk);
    }

    #[test]
    fn ollama_stream_chunk_deserialises_stop_stream_message() {
        // arrange
        let message_json_str = r#"
{
  "id": "chatcmpl-361",
  "object": "chat.completion.chunk",
  "created": 1779505726,
  "model": "phi4-mini:3.8b",
  "system_fingerprint": "fp_ollama",
  "choices": [
    { "index": 0, "delta": { "role": "assistant", "content": "" }, "finish_reason": "stop" }
  ]
}"#;

        // act
        let chunk: OllamaStreamChunkObject = serde_json::from_str(message_json_str).unwrap();

        // assert
        insta::assert_json_snapshot!(chunk);
    }

    #[test]
    fn ollama_stream_chunk_deserialises_usage_stream_message() {
        // arrange
        let message_json_str = r#"
{
  "id": "chatcmpl-146",
  "object": "chat.completion.chunk",
  "created": 1779640603,
  "model": "qwen3-vl:8b",
  "system_fingerprint": "fp_ollama",
  "choices": [],
  "usage": { "prompt_tokens": 29, "completion_tokens": 253, "total_tokens": 282 }
}"#;

        // act
        let chunk: OllamaStreamChunkObject = serde_json::from_str(message_json_str).unwrap();

        // assert
        insta::assert_json_snapshot!(chunk);
    }
}
