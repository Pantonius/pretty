use std::{error::Error, fmt::Display};

use reqwest::StatusCode;

/* -------------------------------- *
 *  ERRORS                          *
 * -------------------------------- */
#[derive(Debug, Clone)]
pub enum DownloadError {
    Reqwest(String),
    StatusCode(StatusCode),
    IO(String),
}
impl Error for DownloadError {}
impl Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DownloadError occurred: {}",
            match self {
                DownloadError::Reqwest(err) => format!("Failed to request content url \"{}\"", err),
                DownloadError::StatusCode(code) => format!("Unexpected status code: {}", code),
                DownloadError::IO(err) => format!("Couldn't save to file \"{}\"", err),
            }
        )
    }
}
impl From<reqwest::Error> for DownloadError {
    fn from(value: reqwest::Error) -> Self {
        DownloadError::Reqwest(value.to_string())
    }
}
impl From<std::io::Error> for DownloadError {
    fn from(value: std::io::Error) -> Self {
        DownloadError::IO(value.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum CompilationError {
    OSUnsupported,
    Pandoc(String),
    FileNotFound,
}
impl Error for CompilationError {}
impl Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CompilationError occurred: {}",
            match self {
                CompilationError::Pandoc(err) => format!("Failed to run pandoc: {}", err),
                _ => self.to_string(),
            }
        )
    }
}
impl From<std::io::Error> for CompilationError {
    fn from(value: std::io::Error) -> Self {
        CompilationError::Pandoc(value.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum PrettyError {
    ConfigDirNotFound,
    Initialization(String),
    Download(DownloadError),
    Compilation(CompilationError),
}
impl Display for PrettyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "An Error occurred during execution: {}",
            match self {
                PrettyError::ConfigDirNotFound => "Config directory could not be found".to_string(),
                PrettyError::Initialization(err_msg) => err_msg.to_string(),
                PrettyError::Download(err) => err.to_string(),
                PrettyError::Compilation(err) => err.to_string(),
            }
        )
    }
}
impl From<DownloadError> for PrettyError {
    fn from(value: DownloadError) -> Self {
        PrettyError::Download(value)
    }
}
impl From<CompilationError> for PrettyError {
    fn from(value: CompilationError) -> Self {
        PrettyError::Compilation(value)
    }
}
impl From<std::io::Error> for PrettyError {
    fn from(value: std::io::Error) -> Self {
        PrettyError::Initialization(value.to_string())
    }
}
