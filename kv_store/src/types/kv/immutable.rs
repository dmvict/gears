use std::{convert::Infallible, ops::Bound};

use database::Database;

use crate::{
    range::Range,
    types::{prefix::immutable::ImmutablePrefixStore, query::kv::QueryKVStore},
    ApplicationStore, QueryableKVStore, TransactionStore,
};

use super::KVBank;

/// Internal structure which holds different stores
#[derive(Debug)]
pub(crate) enum KVStoreBackend<'a, DB> {
    Commit(&'a KVBank<DB, ApplicationStore>),
    Cache(&'a KVBank<DB, TransactionStore>),
    Query(&'a QueryKVStore<DB>),
}

/// Non mutable kv store
#[derive(Debug)]
pub struct KVStore<'a, DB>(pub(crate) KVStoreBackend<'a, DB>);

impl<'a, DB: Database> KVStore<'a, DB> {
    pub fn range(
        &self,
        range: (Bound<Vec<u8>>, Bound<Vec<u8>>),
    ) -> Range<'a, (Bound<Vec<u8>>, Bound<Vec<u8>>), DB> {
        match self.0 {
            KVStoreBackend::Commit(var) => var.range(range),
            KVStoreBackend::Cache(var) => var.range(range),
            KVStoreBackend::Query(var) => var.range(range),
        }
    }
}

impl<'a, DB: Database> QueryableKVStore for KVStore<'a, DB> {
    type Prefix = ImmutablePrefixStore<'a, DB>;

    type Err = Infallible;

    fn get<R: AsRef<[u8]> + ?Sized>(&self, k: &R) -> Result<Option<Vec<u8>>, Self::Err> {
        Ok(match self.0 {
            KVStoreBackend::Commit(var) => var.get(k),
            KVStoreBackend::Cache(var) => var.get(k),
            KVStoreBackend::Query(var) => var.get(k),
        })
    }

    fn prefix_store<I: IntoIterator<Item = u8>>(self, prefix: I) -> Self::Prefix {
        match self.0 {
            KVStoreBackend::Commit(var) => var.prefix_store(prefix),
            KVStoreBackend::Cache(var) => var.prefix_store(prefix),
            KVStoreBackend::Query(var) => var.prefix_store(prefix),
        }
    }
}

impl<'a, DB> From<&'a KVBank<DB, ApplicationStore>> for KVStore<'a, DB> {
    fn from(value: &'a KVBank<DB, ApplicationStore>) -> Self {
        Self(KVStoreBackend::Commit(value))
    }
}

impl<'a, DB> From<&'a KVBank<DB, TransactionStore>> for KVStore<'a, DB> {
    fn from(value: &'a KVBank<DB, TransactionStore>) -> Self {
        Self(KVStoreBackend::Cache(value))
    }
}

impl<'a, DB> From<&'a QueryKVStore<DB>> for KVStore<'a, DB> {
    fn from(value: &'a QueryKVStore<DB>) -> Self {
        Self(KVStoreBackend::Query(value))
    }
}
