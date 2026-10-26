use alexia::{
    providers::{LanguageModel, antigravity::AntigravityProvider},
    voice::normalize_for_voice,
};
use std::{env, path::PathBuf, process::ExitCode};

#[tokio::main]
async fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments.len() != 1 {
        eprintln!("Uso: cargo run --example ask -- \"Tu pregunta\"");
        return ExitCode::FAILURE;
    }
    let executable = env::var_os("AGY_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(if cfg!(windows) { "agy.exe" } else { "agy" }));
    // A separate workspace keeps repository files out of the normal CLI context.
    let workspace = match env::current_dir() {
        Ok(path) => path.join("target").join("antigravity-local"),
        Err(error) => {
            eprintln!("No se pudo localizar el proyecto: {error}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(error) = std::fs::create_dir_all(&workspace) {
        eprintln!("No se pudo crear el directorio local: {error}");
        return ExitCode::FAILURE;
    }
    let provider = AntigravityProvider::new(executable, workspace);
    match provider.generate(&arguments[0]).await {
        Ok(answer) => {
            let speech = normalize_for_voice(&answer, 800);
            if speech.is_empty() {
                eprintln!("La respuesta no contiene texto apto para voz");
                return ExitCode::FAILURE;
            }
            println!("{speech}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
