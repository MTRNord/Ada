use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    /// Represents all cases of `serde_yaml::Error`.
    #[error(transparent)]
    SerdeError(#[from] serde_yaml::Error),

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum SpeechError {
    #[error("Model couldn't be loaded")]
    ModelNotFound,
}
