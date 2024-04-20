use bevy::utils::thiserror::{self,
                             Error};

#[derive(Debug, Error)]
pub enum PakReadErr {
    #[error("PAKDIR error")]
    DeserializeError(#[from] bincode::Error),
    #[error("IO error")]
    IOError(#[from] std::io::Error),
}
