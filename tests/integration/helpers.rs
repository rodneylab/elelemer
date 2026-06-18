use std::net::{IpAddr, Ipv4Addr};

use elelemer::{
    configuration::{ApiConfig, UserApiConfig},
    errors::AppError,
    runners::{LlmRunnerParser, Runner},
};
use launchdarkly_sdk_transport::HeaderValue;
use mocktail::{
    MockSet,
    server::{MockServer, MockServerConfig},
};
use secrecy::SecretBox;

pub struct TestRunner {}

pub enum TestRunnerType {
    Llamacpp,
    Ollama,
}

impl TestRunner {
    pub fn build_mocks<const L: usize>(
        api_key: &str,
        response_stream_chunks: [&'static str; L],
    ) -> MockSet {
        let mut mocks = MockSet::new();
        mocks.mock(|when, then| {
            when.post()
                .path("/v1/chat/completions")
                .headers(TestRunner::request_headers(api_key));
            then.headers(TestRunner::response_headers())
                .bytes_stream(response_stream_chunks);
        });

        mocks
    }

    pub async fn spawn_llm_server(mocks: MockSet, runner: TestRunnerType) -> (MockServer, String) {
        let server = MockServer::new_http(match runner {
            TestRunnerType::Llamacpp => "llama.cpp",
            TestRunnerType::Ollama => "ollama",
        })
        .with_config(MockServerConfig {
            listen_addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
            ..MockServerConfig::default()
        })
        .with_mocks(mocks);
        server.start().await.unwrap();

        let base_url = server.url("/");
        let base_url = base_url.as_str();
        let base_url = base_url.strip_suffix('/').unwrap_or(base_url);

        (server, base_url.to_owned())
    }

    pub fn request_headers(api_key: &str) -> mocktail::Headers {
        let mut request_headers = mocktail::Headers::new();
        request_headers.insert(http::header::ACCEPT, "text/event-stream");
        request_headers.insert(http::header::CACHE_CONTROL, "no-cache");
        request_headers.insert(http::header::CONTENT_TYPE, "application/json");
        request_headers.insert(http::header::AUTHORIZATION, format!("Bearer {api_key}"));

        request_headers
    }

    pub fn response_headers() -> mocktail::Headers {
        let mut response_headers = mocktail::Headers::new();
        response_headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_str("text/event-stream").unwrap(),
        );
        response_headers.insert(
            http::header::TRANSFER_ENCODING,
            HeaderValue::from_str("chunked").unwrap(),
        );

        response_headers
    }
}

pub async fn test_runner_invoke<P: LlmRunnerParser>(
    api_key: &str,
    mock_server_base_url: String,
    parser: P,
    output: &mut Vec<u8>,
) -> Result<(), AppError> {
    let user_settings = UserApiConfig {
        base_url: mock_server_base_url,
        api_key: SecretBox::from(api_key),
        timeout_secs: 600,
    };
    let system_prompt = "You are a helpful assistant";
    let settings = ApiConfig::try_from(user_settings)?;
    let runner = Runner::<P>::new(parser, &settings)?;
    let model = "qxz-supreme-sota:2t";
    let prompt = "How high is the sky?";

    runner.invoke(output, system_prompt, model, prompt).await
}
