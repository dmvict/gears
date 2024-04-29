use store_crate::{database::Database, StoreKey};
use tendermint::types::proto::event::Event;

use super::ExecutionMode;
use crate::{
    application::handlers::node::ABCIHandler,
    baseapp::{errors::RunTxError, genesis::Genesis},
    types::{
        context::{tx::TxContext, TransactionalContext},
        gas::{
            basic_meter::BasicGasMeter, infinite_meter::InfiniteGasMeter, kind::BlockKind, Gas,
            GasMeter,
        },
        tx::{raw::TxWithRaw, TxMessage},
    },
};

use store_crate::TransactionalMultiKVStore;

#[derive(Debug)]
pub struct CheckTxMode {
    pub(crate) block_gas_meter: GasMeter<BlockKind>,
}

impl CheckTxMode {
    pub fn new(max_gas: Gas) -> Self {
        Self {
            block_gas_meter: GasMeter::new(match max_gas {
                Gas::Infinite => Box::new(InfiniteGasMeter::default()),
                Gas::Finite(max_gas) => Box::new(BasicGasMeter::new(max_gas)),
            }),
        }
    }
}

impl<DB: Database, SK: StoreKey> ExecutionMode<DB, SK> for CheckTxMode {
    fn run_msg<'m, M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        ctx: &mut TxContext<'_, DB, SK>,
        _handler: &AH,
        _msgs: impl Iterator<Item = &'m M>,
    ) -> Result<Vec<Event>, RunTxError> {
        ctx.multi_store_mut().tx_caches_clear();

        Ok(ctx.events_drain())
    }

    fn run_ante_checks<M: TxMessage, G: Genesis, AH: ABCIHandler<M, SK, G>>(
        ctx: &mut TxContext<'_, DB, SK>,
        handler: &AH,
        tx_with_raw: &TxWithRaw<M>,
    ) -> Result<(), RunTxError> {
        let result = handler.run_ante_checks(ctx, tx_with_raw);

        ctx.multi_store_mut().tx_caches_clear();

        result.map_err(|e| RunTxError::Custom(e.to_string()))
    }

    fn runnable(&self, _: &mut TxContext<'_, DB, SK>) -> Result<(), RunTxError> {
        Ok(())
    }

    fn block_gas_meter_mut(&mut self) -> &mut GasMeter<BlockKind> {
        &mut self.block_gas_meter
    }

    // fn build_ctx<M: TxMessage>(
    //     &mut self,
    //     multi_store : &mut MultiStore<DB, SK>,
    //     height: u64,
    //     header: &Header,
    //     tx: &TxWithRaw<M>,
    // ) -> TxContext<'_, DB, SK> {
    //     let mut ctx = TxContext::new(
    //         &mut self.multi_store,
    //         height,
    //         header.clone(),
    //         self.block_gas_meter.clone(),
    //     );
    //     ctx.gas_meter
    //         .replace_meter(Self::build_tx_gas_meter(&tx.tx.auth_info.fee, height));

    //     ctx
    // }
}