pub use thiserror::Error;

#[derive(Error, Debug)]
// #[derive(Debug)]
pub enum KvStoreError {
    #[error("Key not found")]
    NotFoundKey,
}

#[derive(Error, Debug)]
pub enum ProcError {
    #[error("Bad len")]
    BadLen,
}
