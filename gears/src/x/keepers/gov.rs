use database::Database;
use kv_store::StoreKey;

use crate::{
    context::QueryableContext,
    types::{
        address::AccAddress, base::coin::UnsignedCoin, denom::Denom,
        store::gas::errors::GasStoreErrors,
    },
    x::module::Module,
};

use super::bank::BankKeeper;

pub trait GovernanceBankKeeper<SK: StoreKey, M: Module>: BankKeeper<SK, M> {
    fn balance_all<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        address: &AccAddress,
    ) -> Result<Vec<UnsignedCoin>, GasStoreErrors>;

    fn balance<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        address: &AccAddress,
        denom: &Denom,
    ) -> Result<UnsignedCoin, GasStoreErrors>;
}
