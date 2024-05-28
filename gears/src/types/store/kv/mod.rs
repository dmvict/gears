pub mod mutable;

use std::ops::RangeBounds;

use database::Database;
use store_crate::{types::kv::immutable::KVStore, QueryableKVStore};

use crate::types::{
    auth::gas::Gas,
    gas::{config::GasConfig, kind::TxKind, GasMeter},
};

use super::{
    constants::{READ_COST_FLAT_DESC, READ_PER_BYTE_DESC},
    errors::GasStoreErrors,
    prefix::GasStorePrefix,
    range::GasRange,
};

#[derive(Debug)]
pub struct GasKVStore<'a, DB> {
    pub(super) gas_meter: &'a mut GasMeter<TxKind>,
    pub(super) inner: KVStore<'a, DB>,
}

impl<'a, DB> GasKVStore<'a, DB> {
    pub(crate) fn new(gas_meter: &'a mut GasMeter<TxKind>, inner: KVStore<'a, DB>) -> Self {
        Self { gas_meter, inner }
    }
}

impl<'a, DB: Database> GasKVStore<'a, DB> {
    pub fn get<R: AsRef<[u8]>>(&mut self, k: R) -> Result<Vec<u8>, GasStoreErrors> {
        self.gas_meter
            .consume_gas(GasConfig::kv().read_cost_flat, READ_COST_FLAT_DESC)?;

        let value = self.inner.get(&k);

        let read_cost_per_byte = GasConfig::kv().read_cost_per_byte;

        self.gas_meter.consume_gas(
            read_cost_per_byte
                .checked_mul(Gas::try_from(k.as_ref().len() as u64)?)
                .ok_or(GasStoreErrors::GasOverflow)?,
            READ_PER_BYTE_DESC,
        )?;

        // TODO:NOW is it okay to ignore if value not found. Don't see any other way
        if let Some(ref value) = value {
            self.gas_meter.consume_gas(
                read_cost_per_byte
                    .checked_mul(Gas::try_from(value.len() as u64)?)
                    .ok_or(GasStoreErrors::GasOverflow)?,
                READ_PER_BYTE_DESC,
            )?;
        }

        value.ok_or(GasStoreErrors::NotFound)
    }

    pub fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> GasStorePrefix<'a, DB> {
        GasStorePrefix::new(self, prefix)
    }

    pub fn range<R: RangeBounds<Vec<u8>> + Clone>(&mut self, range: R) -> GasRange<'_, R, DB> {
        GasRange::new(self.inner.range(range), &mut self.gas_meter)
    }
}
