use alexia::providers::{LanguageModel, ProviderError, antigravity::AntigravityProvider};
use std::path::PathBuf;

#[tokio::test]
async fn local_provider_rejects_empty_and_oversized_questions_before_starting_cli() {
    let provider = AntigravityProvider::new(
        PathBuf::from("missing-antigravity-executable"),
        std::env::current_dir().unwrap(),
    );
    for prompt in ["  ".to_owned(), "a".repeat(4001)] {
        assert_eq!(
            provider.generate(&prompt).await,
            Err(ProviderError::InvalidPrompt)
        );
    }
}

#[tokio::test]
async fn missing_cli_returns_an_actionable_error_without_credentials() {
    let provider = AntigravityProvider::new(
        PathBuf::from("missing-antigravity-executable"),
        std::env::current_dir().unwrap(),
    );
    assert_eq!(
        provider.generate("Hola").await,
        Err(ProviderError::CliUnavailable)
    );
}
