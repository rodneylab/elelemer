use std::time::Duration;

use http::HeaderValue;
use launchdarkly_sdk_transport::HyperTransport;
use secrecy::{ExposeSecret, SecretBox};
use url::Url;

use crate::{
    errors::{RunnerStreamInitialisationError, UnexpectedError},
    openai_api::OpenAIRequest,
};

const UNEXPECTED_ISSUE_ADVICE: &str = "Unexpected error.  Please rerun in debug mode and provide \
log details when raising an issue.";

fn map_header_parse_error(err: eventsource_client::Error) -> RunnerStreamInitialisationError {
    RunnerStreamInitialisationError::Unexpected(UnexpectedError {
        advice: UNEXPECTED_ISSUE_ADVICE.to_owned(),
        context: "parsing runner request headers".to_owned(),
        cause: err.to_string(),
    })
}

/// Client providing Server-Sent Event connections, for connecting to LLM runners.
#[derive(Debug)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct SseClient {
    /// Timeout for awaiting an event, most valuable for longer prompts, for which there may be a
    /// delay before the first chunk of the response stream.
    pub read_timeout: Duration,

    /// Url for connecting to runner API
    pub url: Url,
}

fn map_build_transport_error(err: std::io::Error) -> RunnerStreamInitialisationError {
    RunnerStreamInitialisationError::Unexpected(UnexpectedError {
        advice: UNEXPECTED_ISSUE_ADVICE.to_owned(),
        context: "initialising HTTPS transport client".to_owned(),
        cause: err.to_string(),
    })
}

fn map_json_parse_error(err: serde_json::Error) -> RunnerStreamInitialisationError {
    RunnerStreamInitialisationError::Unexpected(UnexpectedError {
        advice: "Unexpected error.  Please contact support.".to_owned(),
        context: "serialising runner request body".to_owned(),
        cause: err.to_string(),
    })
}

impl SseClient {
    /// Build an HTTP transport for the SSE Client
    ///
    /// Should be infallible when using the `hyper-rustls-webpki-roots` feature on the
    /// `launchdarkly_sdk_transport` crate.
    pub fn build_transport(
        &self,
    ) -> Result<impl launchdarkly_sdk_transport::HttpTransport, RunnerStreamInitialisationError>
    {
        HyperTransport::builder()
            .connect_timeout(std::time::Duration::from_secs(10))
            .read_timeout(self.read_timeout)
            .build_https()
            .map_err(map_build_transport_error)
    }

    /// Generate a client instance
    pub fn build_client<'a>(
        &self,
        api_key: &SecretBox<str>,
        request_data: OpenAIRequest<'a>,
        transport: impl launchdarkly_sdk_transport::HttpTransport,
    ) -> Result<impl eventsource_client::Client, RunnerStreamInitialisationError> {
        debug_assert!(eventsource_client::ClientBuilder::for_url(self.url.as_str()).is_ok());
        debug_assert!(
            HeaderValue::from_str(&format!("Bearer {}", api_key.expose_secret())).is_ok()
        );

        Ok(
            eventsource_client::ClientBuilder::for_url(self.url.as_str())
                .expect("Url should be valid")
                .method("POST".to_string())
                .header("content-type", "application/json")
                .map_err(map_header_parse_error)?
                .header(
                    "authorization",
                    &format!("Bearer {}", api_key.expose_secret()),
                )
                .expect("Should be a valid header")
                .body(serde_json::to_string(&request_data).map_err(map_json_parse_error)?)
                .build_with_transport(transport),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::io::{ErrorKind, Read};

    use crate::server_sent_events::{
        map_build_transport_error, map_header_parse_error, map_json_parse_error,
    };

    #[test]
    fn map_header_parse_error_generates_expected_value() {
        // arrange
        let error = eventsource_client::Error::TimedOut;

        // act
        let outcome = map_header_parse_error(error);

        // assert
        insta::assert_snapshot!(format!("{outcome:?}"));
    }

    #[test]
    fn map_build_transport_error_generates_expected_value() {
        // arrange
        let error = std::io::Error::other("sorry about this!");

        // act
        let outcome = map_build_transport_error(error);

        // assert
        insta::assert_snapshot!(format!("{outcome:?}"));
    }

    struct ErroringReader();

    impl Read for ErroringReader {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(ErrorKind::TimedOut, "time's up!"))
        }
    }

    #[test]
    fn map_json_parse_error_generates_expected_value() {
        // arrange
        let reader = ErroringReader();
        let error = serde_json::from_reader::<ErroringReader, String>(reader).unwrap_err();

        // act
        let outcome = map_json_parse_error(error);

        // assert
        insta::assert_snapshot!(format!("{outcome:?}"));
    }
}
