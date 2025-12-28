use thiserror::Error;

#[derive(Error, Debug)]
pub enum VideoError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Scene processing error: {0}")]
    SceneError(String),

    #[error("Video generation error: {0}")]
    VideoGenerationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Environment variable error: {0}")]
    EnvError(String),

    #[error("FFmpeg error: {0}")]
    FfmpegError(String),
}

pub type Result<T> = std::result::Result<T, VideoError>;
