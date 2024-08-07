use gears::{
    application::handlers::node::{ModuleInfo, TxError},
    x::errors::BankKeeperError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BankTxError {
    #[error(transparent)]
    Keeper(#[from] BankKeeperError),
}

impl BankTxError {
    pub fn into<MI: ModuleInfo>(self) -> TxError {
        let code = match &self {
            BankTxError::Keeper(_) => nz::u16!(1),
        };

        TxError::new::<MI>(self.to_string(), code)
    }
}
