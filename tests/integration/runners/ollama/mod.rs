mod fixtures;

use elelemer::runners::ollama::OllamaRunner;

use crate::helpers::{TestRunner, TestRunnerType, test_runner_invoke};

#[tokio::test]
async fn ollama_runner_returns_expected_result_for_valid_input() {
    // arrange
    let ollama_api_key = "ollama";
    let mocks = TestRunner::build_mocks(ollama_api_key, fixtures::OLLAMA_DATA_JSON_CHUNKS);
    let (ollama_server, base_url) =
        TestRunner::spawn_llm_server(mocks, TestRunnerType::Ollama).await;
    let mut output = Vec::<u8>::new();

    // act
    assert!(ollama_server.is_running());
    let outcome =
        test_runner_invoke(ollama_api_key, base_url, OllamaRunner::new(), &mut output).await;

    // assert
    assert!(outcome.is_ok());
    insta::assert_snapshot!(std::str::from_utf8(&output).unwrap());
}
