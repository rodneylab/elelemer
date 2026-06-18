#![warn(clippy::all, clippy::pedantic)]

mod helpers;
mod runners;

use snapbox::cmd::Command;

#[test]
fn it_prints_help() {
    // act
    let output = Command::cargo_bin("elelemer")
        .arg("--help")
        .output()
        .unwrap();

    // assert
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
}

#[test]
fn it_handles_happy_path_as_expected() {
    // act
    let output = Command::cargo_bin("elelemer")
        .env("ELELEMER_OLLAMA__TIMEOUT_SECS", "3")
        .arg("ollama")
        .arg("run")
        .arg("gemma3:270m")
        .arg("hi")
        .output()
        .unwrap();

    // assert
    assert!(
        output.stderr.is_empty(),
        "Expected no stderr content; got {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
    assert!(
        stdout.contains("hi") | stdout.contains("help") | stdout.contains("what can i do for you"),
        "Expected output to contain 'help' or 'hi'; got {stdout}",
    );
    assert!(output.status.success());
}
