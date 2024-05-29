use database::prefix::PrefixDB;
use database::Database;
use kv_store::types::kv::immutable::KVStore;
use kv_store::types::{kv::mutable::KVStoreMut, multi::MultiBank};
use kv_store::{ApplicationStore, StoreKey};
use tendermint::types::{chain_id::ChainId, proto::event::Event, time::Timestamp};

use super::{QueryableContext, TransactionalContext};
// use crate::tendermint::types::time::Timestamp;

#[derive(Debug)]
pub struct InitContext<'a, DB, SK> {
    multi_store: &'a mut MultiBank<DB, SK, ApplicationStore>,
    pub(crate) height: u64,
    pub(crate) time: Timestamp,
    pub events: Vec<Event>,
    pub(crate) chain_id: ChainId,
}

impl<'a, DB, SK> InitContext<'a, DB, SK> {
    pub fn new(
        multi_store: &'a mut MultiBank<DB, SK, ApplicationStore>,
        height: u64,
        time: Timestamp,
        chain_id: ChainId,
    ) -> Self {
        InitContext {
            multi_store,
            height,
            time,
            events: Vec::new(),
            chain_id,
        }
    }

    pub fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }
}

impl<'a, DB: Database, SK: StoreKey> InitContext<'a, DB, SK> {
    pub fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        KVStore::from(self.multi_store.kv_store(store_key))
    }

    pub fn kv_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, PrefixDB<DB>> {
        KVStoreMut::from(self.multi_store.kv_store_mut(store_key))
    }
}

impl<DB: Database, SK: StoreKey> QueryableContext<DB, SK> for InitContext<'_, DB, SK> {
    fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        self.kv_store(store_key)
    }

    fn height(&self) -> u64 {
        self.height
    }
}

impl<DB: Database, SK: StoreKey> TransactionalContext<DB, SK> for InitContext<'_, DB, SK> {
    fn kv_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, PrefixDB<DB>> {
        self.kv_store_mut(store_key)
    }

    fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    fn append_events(&mut self, mut events: Vec<Event>) {
        self.events.append(&mut events);
    }

    fn events_drain(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    fn get_time(&self) -> Option<Timestamp> {
        Some(self.time.clone())
    }
}
