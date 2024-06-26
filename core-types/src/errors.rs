#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("invalid coins: `{0}`")]
    Coins(String),
    #[error("invalid coin: `{0}`")]
    Coin(String),
    #[error(transparent)]
    Decode(#[from] prost::DecodeError),
    #[error("{0}")]
    DecodeProtobuf(String),
    #[error("decode error: `{0}`")]
    DecodeAny(String),
    #[error("missing field: `{0}`")]
    MissingField(String),
    #[error("decode error: `{0}`")]
    DecodeAddress(String),
    #[error("decode error: `{0}`")]
    DecodeGeneral(String),
    #[error("serde error: {0}")]
    SerdeSerialize(String),
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    Custom(String),
    #[error("invalid sign mode: `{0}`")]
    InvalidSignMode(i32),
}

impl From<CoreError> for tonic::Status {
    fn from(e: CoreError) -> Self {
        tonic::Status::invalid_argument(format!("{:?}", e))
    }
}
