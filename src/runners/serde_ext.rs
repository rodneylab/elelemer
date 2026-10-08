use serde::{Deserialize, Deserializer};

use crate::runners::{
    OpenAiCompatibleStreamChunkChoiceDeltaContent,
    OpenAiCompatibleStreamChunkChoiceDeltaReasoningContent,
    llamacpp::{
        LlamacppStreamChunkChoiceDelta, LlamacppStreamChunkChoiceDeltaReasoningContent,
        LlamacppStreamChunkChoiceDeltaRoleContent,
    },
    ollama::OllamaStreamChunkChoiceDelta,
};

pub fn deserialise_with_empty_delta<'de, D>(
    deserializer: D,
) -> Result<Option<LlamacppStreamChunkChoiceDelta>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, serde::Deserialize)]
    #[cfg_attr(test, derive(serde::Serialize))]
    #[serde(
        untagged,
        deny_unknown_fields,
        expecting = "object, empty object or null"
    )]
    pub enum Helper {
        Empty {},
        RoleContent(LlamacppStreamChunkChoiceDeltaRoleContent),
        Content(OpenAiCompatibleStreamChunkChoiceDeltaContent),
        ReasoningContent(LlamacppStreamChunkChoiceDeltaReasoningContent),
    }

    match Helper::deserialize(deserializer) {
        Ok(Helper::Empty {}) => Ok(None),
        Ok(Helper::RoleContent(value)) => {
            Ok(Some(LlamacppStreamChunkChoiceDelta::RoleContent(value)))
        }
        Ok(Helper::Content(value)) => Ok(Some(LlamacppStreamChunkChoiceDelta::Content(value))),
        Ok(Helper::ReasoningContent(value)) => Ok(Some(
            LlamacppStreamChunkChoiceDelta::ReasoningContent(value),
        )),
        Err(e) => Err(e),
    }
}

pub fn deserialise_ollama_open_api_helper<'de, D>(
    deserializer: D,
) -> Result<Option<OllamaStreamChunkChoiceDelta>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, serde::Deserialize)]
    #[cfg_attr(test, derive(serde::Serialize))]
    pub struct Helper {
        content: Option<String>,
        reasoning: Option<String>,
        role: Option<String>,
    }

    match Helper::deserialize(deserializer) {
        Ok(Helper {
            content,
            reasoning,
            role,
        }) => {
            if let Some(value) = role
                && value != "assistant"
            {
                log::warn!("Unexpected non-assistant role in response");

                return Ok(None);
            }
            if let Some(value) = reasoning
                && !value.is_empty()
            {
                Ok(Some(OllamaStreamChunkChoiceDelta::ReasoningContent(
                    OpenAiCompatibleStreamChunkChoiceDeltaReasoningContent { reasoning: value },
                )))
            } else {
                Ok(Some(OllamaStreamChunkChoiceDelta::Content(
                    OpenAiCompatibleStreamChunkChoiceDeltaContent {
                        content: content.unwrap_or_default(),
                    },
                )))
            }
        }
        Err(e) => Err(e),
    }
}
