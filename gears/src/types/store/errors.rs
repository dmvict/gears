use crate::types::{auth::gas::GasError, gas::GasMeteringErrors};

#[derive(Debug, Clone, thiserror::Error)]
pub enum GasStoreErrors {
    #[error("key not found")]
    NotFound,
    #[error("gas overflow")]
    GasOverflow,
    #[error("Metering error: {0}")]
    Metering(#[from] GasMeteringErrors),
    #[error("Gas error: {0}")]
    Gas(#[from] GasError),
}
