pub mod gemini;

use std::{error::Error, fmt, future::Future, pin::Pin};

pub type Generation<'a> = Pin<Box<dyn Future<Output = Result<String, ProviderError>> + Send + 'a>>;

/// Provider boundary independent of Alexa, HTTP and voice formatting.
pub trait LanguageModel: Send + Sync {
    fn generate<'a>(&'a self, prompt: &'a str) -> Generation<'a>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProviderError {
    NotConfigured,
}

impl fmt::Display for ProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConfigured => formatter.write_str("language model provider is not configured"),
        }
    }
}

impl Error for ProviderError {}
