pub mod antigravity;
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
    InvalidPrompt,
    CliUnavailable,
    CliFailed,
    TimedOut,
    InvalidResponse,
    OutputTooLarge,
}

impl fmt::Display for ProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConfigured => formatter.write_str("language model provider is not configured"),
            Self::InvalidPrompt => {
                formatter.write_str("la pregunta debe tener entre 1 y 4000 caracteres")
            }
            Self::CliUnavailable => formatter.write_str(
                "no se pudo iniciar Antigravity; revisa AGY_BIN y su directorio de trabajo",
            ),
            Self::CliFailed => {
                formatter.write_str("Antigravity fallo; abre agy y comprueba tu sesion y cuota")
            }
            Self::TimedOut => formatter.write_str("Antigravity excedio el limite de tiempo"),
            Self::InvalidResponse => {
                formatter.write_str("Antigravity no devolvio una respuesta JSON valida")
            }
            Self::OutputTooLarge => {
                formatter.write_str("la respuesta de Antigravity excedio el limite permitido")
            }
        }
    }
}

impl Error for ProviderError {}
