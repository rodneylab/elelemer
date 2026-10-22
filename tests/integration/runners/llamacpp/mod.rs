mod fixtures;

use elelemer::runners::llamacpp::LlamaCppRunner;

use crate::helpers::{TestRunner, TestRunnerType, test_runner_invoke};

#[tokio::test]
async fn llamacpp_runner_returns_expected_result_for_valid_input() {
    // arrange
    let llamacpp_api_key = "llamacpp";
    let mocks = TestRunner::build_mocks(llamacpp_api_key, fixtures::LLAMACPP_DATA_JSON_CHUNKS);
    let (_llamacpp_server, base_url) =
        TestRunner::spawn_llm_server(mocks, TestRunnerType::Llamacpp).await;
    let mut output = Vec::<u8>::new();

    // act
    let outcome = test_runner_invoke(
        llamacpp_api_key,
        base_url,
        LlamaCppRunner::new(),
        &mut output,
    )
    .await;

    // assert
    assert!(outcome.is_ok());
    insta::assert_snapshot!(std::str::from_utf8(&output).unwrap());
}
