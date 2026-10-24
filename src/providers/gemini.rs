use super::{Generation, LanguageModel, ProviderError};

/// Placeholder only: does not load credentials or make network requests.
pub struct GeminiProvider;

impl LanguageModel for GeminiProvider {
    fn generate<'a>(&'a self, _prompt: &'a str) -> Generation<'a> {
        Box::pin(async { Err(ProviderError::NotConfigured) })
    }
}
