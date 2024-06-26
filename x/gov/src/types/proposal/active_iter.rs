use std::ops::Bound;

use chrono::{DateTime, SubsecRound, Utc};
use gears::{
    store::database::Database,
    types::store::{gas::errors::GasStoreErrors, kv::Store, range::StoreRange},
};

use crate::errors::SERDE_JSON_CONVERSION;

use super::{parse_proposal_key_bytes, Proposal, SORTABLE_DATE_TIME_FORMAT};

#[derive(Debug)]
pub struct ActiveProposalIterator<'a, DB>(StoreRange<'a, DB>);

impl<'a, DB: Database> ActiveProposalIterator<'a, DB> {
    pub fn new(store: Store<'a, DB>, end_time: &DateTime<Utc>) -> ActiveProposalIterator<'a, DB> {
        Self(
            store.into_range((
                Bound::Included(Proposal::KEY_ACTIVE_QUEUE_PREFIX.to_vec()),
                Bound::Excluded(
                    [
                        Proposal::KEY_ACTIVE_QUEUE_PREFIX.as_slice(),
                        end_time
                            .round_subsecs(0)
                            .format(SORTABLE_DATE_TIME_FORMAT)
                            .to_string()
                            .as_bytes(),
                    ]
                    .concat()
                    .to_vec(),
                ),
            )),
        )
    }
}

impl<'a, DB: Database> Iterator for ActiveProposalIterator<'a, DB> {
    type Item = Result<((u64, DateTime<Utc>), Proposal), GasStoreErrors>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(var) = self.0.next() {
            match var {
                Ok((key, value)) => Some(Ok((
                    parse_proposal_key_bytes(key.as_ref()),
                    serde_json::from_slice(&value).expect(SERDE_JSON_CONVERSION),
                ))),
                Err(err) => Some(Err(err)),
            }
        } else {
            None
        }
    }
}
