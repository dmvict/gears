use std::sync::{Arc, RwLock};

use store_crate::{database::Database, types::multi::MultiStore, StoreKey};
use tendermint::types::proto::event::Event;

use super::ExecutionMode;
use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{errors::RunTxError, genesis::Genesis},
    types::{
        context::{tx::TxContext, TransactionalContext},
        gas::{
            basic_meter::BasicGasMeter, infinite_meter::InfiniteGasMeter, kind::BlockMeterKind,
            Gas, GasMeter,
        },
        header::Header,
        tx::{raw::TxWithRaw, TxMessage},
    },
};

use store_crate::TransactionalMultiKVStore;

#[derive(Debug)]
pub struct CheckTxMode<DB, SK> {
    pub(crate) block_gas_meter: GasMeter<BlockMeterKind>,
    pub(crate) multi_store: MultiStore<DB, SK>,
}

impl<DB: Database, SK: StoreKey> CheckTxMode<DB, SK> {
    pub fn new(multi_store: MultiStore<DB, SK>, max_gas: Gas) -> Self {
        Self {
            block_gas_meter: GasMeter::new(match max_gas > Gas::new(0) {
                true => Arc::new(RwLock::new(Box::new(InfiniteGasMeter::default()))),
                false => Arc::new(RwLock::new(Box::new(BasicGasMeter::new(max_gas)))),
            }),
            multi_store,
        }
    }
}

impl<DB: Database, SK: StoreKey> ExecutionMode<DB, SK> for CheckTxMode<DB, SK> {
    fn run_msg<'m, M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        &mut self,
        ctx: &mut TxContext<'_, DB, SK>,
        _handler: &AH,
        _msgs: impl Iterator<Item = &'m M>,
    ) -> Result<Vec<Event>, RunTxError> {
        ctx.multi_store_mut().tx_caches_clear();

        Ok(ctx.events_drain())
    }

    fn run_ante_checks<M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        &mut self,
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError> {
        let result = handler.run_ante_checks(ctx, tx_with_raw);

        ctx.multi_store_mut().tx_caches_clear();

        result.map_err(|e| RunTxError::Custom(e.to_string()))
    }

    fn runnable(&self, _ctx: &mut TxContext<'_, DB, SK>) -> Result<(), RunTxError> {
        Ok(())
    }

    fn build_ctx(&mut self, height: u64, header: Header) -> TxContext<'_, DB, SK> {
        TxContext::new(
            &mut self.multi_store,
            height,
            header,
            self.block_gas_meter.clone(),
        )
    }
}
