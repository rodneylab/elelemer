use crate::{
    openai_api::{FinalStreamChunk, StreamChunk, Timings},
    runners::{
        LlmRunnerParser, OpenAiCompatibleStreamChunkChoice,
        OpenAiCompatibleStreamChunkChoiceDeltaContent, OpenAiCompatibleStreamChunkObject,
        extract_final_metadata, map_stream_chunk_json_parse_error,
        parse_open_ai_compatible_stream_chunk_string, serde_ext::deserialise_with_empty_delta,
    },
};

/// OpenAI API-compatible response stream chunk message with author role
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct LlamacppStreamChunkChoiceDeltaRoleContent {
    /// Message author role: "system" or "user"
    pub role: String,

    /// Message body
    pub content: Option<String>,
}

/// OpenAI API-compatible response stream chunk reasoning message
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct LlamacppStreamChunkChoiceDeltaReasoningContent {
    /// Message body
    pub reasoning_content: String,
}

///  OpenAI API-compatible response stream chunk delta
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(untagged)]
pub enum LlamacppStreamChunkChoiceDelta {
    ///  OpenAI API-compatible response stream chunk message without content
    Empty {},

    /// OpenAI API-compatible response stream chunk message with role
    RoleContent(LlamacppStreamChunkChoiceDeltaRoleContent),

    /// OpenAI API-compatible response stream chunk message
    Content(OpenAiCompatibleStreamChunkChoiceDeltaContent),

    /// OpenAI API-compatible response stream chunk reasoning message
    ReasoningContent(LlamacppStreamChunkChoiceDeltaReasoningContent),
}

///  OpenAI API-compatible response stream chunk choice
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct LlamacppStreamChunkChoice {
    ///  OpenAI API-compatible response stream chunk finished marker
    pub finish_reason: Option<String>,

    ///  OpenAI API-compatible response stream chunk delta
    #[serde(default, deserialize_with = "deserialise_with_empty_delta")]
    pub delta: Option<LlamacppStreamChunkChoiceDelta>,
}

impl OpenAiCompatibleStreamChunkChoice for &LlamacppStreamChunkChoice {
    fn finish_reason(&self) -> Option<&str> {
        self.finish_reason.as_deref()
    }
}

/// Lllamacpp runner model token usage
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct LlamacppTimings {
    /// Runner model prompt tokens
    pub prompt_n: u32,

    /// Runner model response tokens
    pub predicted_n: u32,
}

impl From<&LlamacppTimings> for Timings {
    fn from(value: &LlamacppTimings) -> Self {
        Self {
            prompt_n: value.prompt_n,
            predicted_n: value.predicted_n,
        }
    }
}

///  OpenAI API-compatible response stream chunk object
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct LlamacppStreamChunkObject {
    ///  OpenAI API-compatible response stream chunk choices
    pub choices: Vec<LlamacppStreamChunkChoice>,

    /// Lllamacpp runner model name or alias
    pub model: String,

    /// Lllamacpp runner model token usage
    pub timings: Option<LlamacppTimings>,
}

impl OpenAiCompatibleStreamChunkObject for &LlamacppStreamChunkObject {
    fn model(&self) -> &str {
        &self.model
    }

    fn timings(&self) -> Option<Timings> {
        self.timings.as_ref().map(Timings::from)
    }
}

///  OpenAI API-compatible response stream chunk
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(untagged)]
pub enum LlamacppStreamChunk {
    ///  OpenAI API-compatible response stream chunk string
    String(String),

    ///  OpenAI API-compatible response stream chunk object
    Object(LlamacppStreamChunkObject),
}

impl From<LlamacppStreamChunk> for StreamChunk {
    fn from(value: LlamacppStreamChunk) -> Self {
        match value {
            LlamacppStreamChunk::String(chunk) => {
                parse_open_ai_compatible_stream_chunk_string(chunk)
            }
            LlamacppStreamChunk::Object(chunk_object) => {
                if let Some(choice) = chunk_object.choices.first() {
                    match &choice.delta {
                        None | Some(LlamacppStreamChunkChoiceDelta::Empty {}) => {
                            let (model, timings) = extract_final_metadata(choice, &chunk_object);

                            Self::Final(FinalStreamChunk { model, timings })
                        }
                        Some(LlamacppStreamChunkChoiceDelta::RoleContent(
                            LlamacppStreamChunkChoiceDeltaRoleContent { role, content },
                        )) => {
                            if role == "system"
                                && let Some(content) = content
                            {
                                Self::Content(content.to_owned())
                            } else {
                                Self::Empty
                            }
                        }
                        Some(LlamacppStreamChunkChoiceDelta::Content(value)) => {
                            Self::Content(value.content.clone())
                        }
                        Some(LlamacppStreamChunkChoiceDelta::ReasoningContent(value)) => {
                            Self::Reasoning(value.reasoning_content.clone())
                        }
                    }
                } else {
                    Self::Empty
                }
            }
        }
    }
}

/// Llamacpp runner
#[derive(Default)]
pub struct LlamaCppRunner;

impl LlmRunnerParser for LlamaCppRunner {
    fn parse_stream_chunk(&self, chunk: &str) -> Option<StreamChunk> {
        let chunk: Option<LlamacppStreamChunk> = match serde_json::from_str(chunk) {
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
        "llama.cpp"
    }
}

impl LlamaCppRunner {
    /// Create a ['LlamaCppRunner']
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use crate::runners::llamacpp::LlamacppStreamChunkObject;

    #[test]
    fn llamacpp_stream_chunk_deserialises_reasoning_stream_message() {
        // arrange
        let message_json_str = r#"
{
  "choices": [{ "finish_reason": null, "index": 0, "delta": { "reasoning_content": "2" } }],
  "created": 1779723855,
  "id": "chatcmpl-QOBZwCuIp41L2w9OcshOgSrY21SY6KhM",
  "model": "qwen-3.5-35b-a3b",
  "system_fingerprint": "b9290-bcfd1989e",
  "object": "chat.completion.chunk"
}"#;

        // act
        let chunk: LlamacppStreamChunkObject = serde_json::from_str(message_json_str).unwrap();

        // assert
        insta::assert_json_snapshot!(chunk);
    }

    #[test]
    fn llamacpp_stream_chunk_deserialises_content_stream_message() {
        // arrange
        let message_json_str = r#"
{
  "choices": [{ "finish_reason": null, "index": 0, "delta": { "content": "**." } }],
  "created": 1779723858,
  "id": "chatcmpl-QOBZwCuIp41L2w9OcshOgSrY21SY6KhM",
  "model": "qwen-3.5-35b-a3b",
  "system_fingerprint": "b9290-bcfd1989e",
  "object": "chat.completion.chunk"
}"#;

        // act
        let chunk: LlamacppStreamChunkObject = serde_json::from_str(message_json_str).unwrap();

        // assert
        insta::assert_json_snapshot!(chunk);
    }

    #[test]
    fn llamacpp_stream_chunk_deserialises_stop_stream_message() {
        // arrange
        let message_json_str = r#"
{
  "choices": [{ "finish_reason": "stop", "index": 0, "delta": {} }],
  "created": 1779723858,
  "id": "chatcmpl-QOBZwCuIp41L2w9OcshOgSrY21SY6KhM",
  "model": "qwen-3.5-35b-a3b",
  "system_fingerprint": "b9290-bcfd1989e",
  "object": "chat.completion.chunk"
}"#;

        // act
        let chunk: LlamacppStreamChunkObject = serde_json::from_str(message_json_str).unwrap();

        // assert
        insta::assert_json_snapshot!(chunk);
    }
}
