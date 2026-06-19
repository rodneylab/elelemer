use std::time::Duration;

use indicatif::ProgressBar;
use secrecy::SecretString;
use url::Url;

#[cfg(test)]
use crate::configuration::serialise_secret;

use crate::{
    configuration::ApiConfig,
    errors::{AppError, RunnerStreamInitialisationError},
    server_sent_events::SseClient,
};

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
/// Request message role type
pub enum Role {
    /// Used for generating system messages
    System,

    /// Used for generating regular user messages
    User,
}

/// Request system or user message
#[derive(Debug, serde::Serialize)]
pub struct Message {
    /// Message author role ("system" or "user")
    pub role: Role,

    /// Message body
    pub content: String,
}

/// OpenAI API stream request options
#[derive(Debug, serde::Serialize)]
pub struct OpenAIRequestStreamOptions {
    /// Whether token usaage should be included in the response stream.
    ///
    /// Typically, the runner provides aggregate token usage for the whole run in the final chunk of
    /// the response stream, rather than a running count in each chunk.
    pub include_usage: bool,
}

/// OpenAI API request
#[derive(Debug, serde::Serialize)]
pub struct OpenAIRequest<'a> {
    /// Runner model
    pub model: &'a str,

    /// Request messages
    pub messages: Vec<Message>,

    /// Whether the server should respond with a stream of events, as thery become available or
    /// buffer everything into a single response
    pub stream: bool,

    /// OpenAI API stream request options
    pub stream_options: OpenAIRequestStreamOptions,
}

/// Response and prompt token usage
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Timings {
    /// Number of token in the prompt
    pub prompt_n: u32,

    /// Number of token in the response
    pub predicted_n: u32,
}

/// Open AI stream final chunk, potentially including timing data and model metadata
#[derive(Debug, Default)]
pub struct FinalStreamChunk {
    /// Model name or alias
    pub model: Option<String>,

    /// Run token usage
    pub timings: Option<Timings>,
}

/// OpenAI API response stream chunk
#[derive(Debug)]
pub enum StreamChunk {
    /// Stream chunk with no content or reasoning
    Empty,

    /// Response chunk
    Content(String),

    /// Reasoning chunk.
    ///
    /// This variant is only constructed when both the model and runner support reasoning.
    Reasoning(String),

    /// Final chunk
    Final(FinalStreamChunk),
}

/// OpenAI API compatible client
#[derive(Debug)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct OpenAIAPIClient<'a> {
    #[cfg(not(test))]
    api_key: &'a SecretString,

    #[cfg(test)]
    #[serde(serialize_with = "serialise_secret")]
    api_key: &'a SecretString,

    sse_client: SseClient,
}

fn completions_url_from_base_url(base_url: &Url) -> Url {
    let mut url = base_url.clone();
    url.set_path("/v1/chat/completions");

    url
}

impl<'a> OpenAIAPIClient<'a> {
    /// Create a new [`OpenAIAPIClient`] instance
    pub fn new(api_config: &'a ApiConfig) -> Result<Self, AppError> {
        let url = completions_url_from_base_url(api_config.base_url());

        let sse_client = SseClient {
            read_timeout: *api_config.timeout(),
            url,
        };

        Ok(Self {
            api_key: api_config.api_key(),
            sse_client,
        })
    }

    fn build_request<'b>(
        &self,
        model: &'b str,
        system_prompt: &str,
        prompt: &str,
    ) -> OpenAIRequest<'b> {
        OpenAIRequest {
            model,
            messages: vec![
                Message {
                    role: Role::System,
                    content: system_prompt.to_owned(),
                },
                Message {
                    role: Role::User,
                    content: prompt.to_owned(),
                },
            ],
            stream: true,
            stream_options: {
                OpenAIRequestStreamOptions {
                    include_usage: true,
                }
            },
        }
    }

    /// Build a runner API server connecting client
    pub fn create_server_events_client(
        &self,
        model: &str,
        system_prompt: &str,
        prompt: &str,
    ) -> Result<impl eventsource_client::Client, RunnerStreamInitialisationError> {
        let bar = ProgressBar::new_spinner();
        bar.enable_steady_tick(Duration::from_millis(100));

        let request_data = self.build_request(model, system_prompt, prompt);
        let transport = self.sse_client.build_transport()?;
        let client = self
            .sse_client
            .build_client(self.api_key, request_data, transport)?;

        bar.finish();

        Ok(client)
    }
}

#[cfg(test)]
mod tests {
    use secrecy::SecretString;
    use url::Url;

    use crate::{
        configuration::{ApiConfig, UserApiConfig},
        openai_api::{
            Message, OpenAIAPIClient, OpenAIRequest, OpenAIRequestStreamOptions, Role,
            completions_url_from_base_url,
        },
    };

    #[test]
    fn completions_url_from_base_url_returns_expected_url_for_base_with_empty_pathname() {
        // arrange
        let url = Url::parse("https://example.com").expect("Should be a valid url");

        // act
        let outcome = completions_url_from_base_url(&url);

        // assert
        assert_eq!(outcome.as_str(), "https://example.com/v1/chat/completions");
    }

    #[test]
    fn completions_url_from_base_url_returns_expected_url_for_base_with_pathname_as_slash() {
        // arrange
        let url = Url::parse("https://example.com/").expect("Should be a valid url");

        // act
        let outcome = completions_url_from_base_url(&url);

        // assert
        assert_eq!(outcome.as_str(), "https://example.com/v1/chat/completions");
    }

    #[test]
    fn completions_url_from_base_url_returns_expected_url_for_base_with_arbitrary_pathname() {
        // arrange
        let url = Url::parse("https://example.com/path/subpath/").expect("Should be a valid url");

        // act
        let outcome = completions_url_from_base_url(&url);

        // assert
        assert_eq!(outcome.as_str(), "https://example.com/v1/chat/completions");
    }

    #[test]
    fn message_serialises_system_message_as_expected() {
        // arrange
        let message = Message {
            role: Role::System,
            content: String::from("Why is the sea blue?"),
        };

        // act
        // assert
        insta::assert_json_snapshot!(message);
    }

    #[test]
    fn message_serialises_user_message_as_expected() {
        // arrange
        let message = Message {
            role: Role::User,
            content: String::from("When did the Mayans live?"),
        };

        // act
        // assert
        insta::assert_json_snapshot!(message);
    }

    #[test]
    fn open_ai_request_serialises_as_expected() {
        // arrange
        let request = OpenAIRequest {
            model: "qxz-supreme-sota:2t",
            messages: vec![
                Message {
                    role: Role::System,
                    content: String::from("You are a helpful assistant"),
                },
                Message {
                    role: Role::User,
                    content: String::from("Did the Mayans live at the same time as the Aztecs?"),
                },
            ],
            stream: true,
            stream_options: OpenAIRequestStreamOptions {
                include_usage: true,
            },
        };

        // act
        // assert
        insta::assert_json_snapshot!(request);
    }

    #[test]
    fn open_ai_api_client_new_returns_expected_value() {
        // arrange
        let config = ApiConfig::try_from(UserApiConfig {
            base_url: String::from("https://example.com"),
            api_key: SecretString::from("abracadabra"),
            timeout_secs: 10,
        })
        .expect("Should be valid configuration");

        // act
        let outcome = OpenAIAPIClient::new(&config).expect("Should successfully generate a client");

        // assert
        insta::assert_json_snapshot!(outcome);
    }

    #[test]
    fn open_ai_api_client_build_request_returns_expected_value() {
        // arrange
        let config = ApiConfig::try_from(UserApiConfig {
            base_url: String::from("https://example.com"),
            api_key: SecretString::from("abracadabra"),
            timeout_secs: 10,
        })
        .expect("Should be valid configuration");
        let client = OpenAIAPIClient::new(&config).expect("Should successfully generate a client");

        // act
        let outcome = client.build_request(
            "qxz-supreme-sota",
            "You are a helpful assistant",
            "Which countries use Fahrenheit for temperatures?",
        );

        // assert
        insta::assert_json_snapshot!(outcome);
    }
}
