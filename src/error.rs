pub use thiserror::Error;

#[derive(Error, Debug)]
// #[derive(Debug)]
pub enum KvStoreError {
    // #[error("io error")]
    // NotFoundKey(#[from] io::Error),
    //  #[error("set error")]
    //  SetError,
    //  #[error("get error")]
    //  GetError,
    #[error("not found key")]
    NotFoundKey,
}
