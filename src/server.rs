use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShutdownError {
    #[error("unkown cause {0}")]
    UnknownCause(String),
}

#[tonic::async_trait]
pub trait Administrable {
    async fn shutdown(&mut self) -> Result<(), ShutdownError>;
}
