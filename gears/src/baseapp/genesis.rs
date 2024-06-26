use crate::{
    error::AppError,
    types::{address::AccAddress, base::send::SendCoins},
};
use serde::{de::DeserializeOwned, Serialize};

pub trait Genesis: Default + DeserializeOwned + Serialize + Clone + Send + Sync + 'static {
    fn add_genesis_account(
        &mut self,
        address: AccAddress,
        coins: SendCoins,
    ) -> Result<(), AppError>;
}
