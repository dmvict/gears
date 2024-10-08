use super::*;
use gears::{
    context::InfallibleContext, core::Protobuf, extensions::corruption::UnwrapCorrupt,
    types::base::coin::Uint256Proto,
};
use prost::{bytes::Bytes, Message};

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: StakingBankKeeper<SK, M>,
        KH: KeeperHooks<SK, AK, M>,
        M: Module,
    > Keeper<SK, PSK, AK, BK, KH, M>
{
    /// Load the last total validator power.
    // TODO: this isn't used
    // TODO: implementation doesn't look right: it's decoding Uint256 from bytes, instead of Uint256Proto
    pub fn last_total_power<DB: Database, CTX: InfallibleContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Option<Uint256> {
        let store = InfallibleContext::infallible_store(ctx, &self.store_key);
        store
            .get(&LAST_TOTAL_POWER_KEY)
            .map(|bytes| Uint256::from_be_bytes(bytes.try_into().unwrap_or_corrupt()))
    }

    pub fn set_last_total_power<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        last_total_power: Uint256,
    ) -> Result<(), GasStoreErrors> {
        let mut store = TransactionalContext::kv_store_mut(ctx, &self.store_key);
        let val = Uint256Proto {
            uint: last_total_power,
        }
        .encode_vec();
        store.set(LAST_TOTAL_POWER_KEY, val)
    }

    /// get the last validator set
    pub fn validators_power_store_vals_vec<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> anyhow::Result<Vec<ValAddress>> {
        let store = ctx.kv_store(&self.store_key);
        let iterator = store.prefix_store(VALIDATORS_BY_POWER_INDEX_KEY);
        let mut res = Vec::new();
        // TODO: we're iterating over every validator here: this method should return an iterator
        for next in iterator.into_range(..) {
            let (_k, v) = next?;
            res.push(ValAddress::try_from(v.to_vec())?);
        }
        Ok(res)
    }

    pub fn set_validator_by_power_index<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> Result<(), GasStoreErrors> {
        let power_reduction = self.power_reduction(ctx);
        let mut store = TransactionalContext::kv_store_mut(ctx, &self.store_key);

        // jailed validators are not kept in the power index
        if validator.jailed {
            return Ok(());
        }

        store.set(
            validator.key_by_power_index_key(power_reduction),
            Vec::from(validator.operator_address.clone()),
        )
    }

    pub fn set_new_validator_by_power_index<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> Result<(), GasStoreErrors> {
        let power_reduction = self.power_reduction(ctx);
        let mut store = ctx.kv_store_mut(&self.store_key);
        store.set(
            validator.key_by_power_index_key(power_reduction),
            Vec::from(validator.operator_address.clone()),
        )
    }

    pub fn delete_validator_by_power_index<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &Validator,
    ) -> Result<Option<Vec<u8>>, GasStoreErrors> {
        let power_reduction = self.power_reduction(ctx);
        let mut store = ctx.kv_store_mut(&self.store_key);
        store.delete(&validator.key_by_power_index_key(power_reduction))
    }

    /// get the last validator set
    pub fn last_validators_by_addr<DB: Database, CTX: InfallibleContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> HashMap<ValAddress, u64> {
        let mut last = HashMap::new();
        let store = ctx.infallible_store(&self.store_key);
        let store = store.prefix_store(LAST_VALIDATOR_POWER_KEY);
        for (k, v) in store.into_range(..) {
            let k = ValAddress::try_from_prefix_length_bytes(&k).unwrap_or_corrupt();
            last.insert(
                k,
                i64::decode(Bytes::copy_from_slice(&v))
                    .unwrap_or_corrupt()
                    .try_into()
                    .unwrap_or_corrupt(),
            );
        }
        last
    }

    /// get the group of the bonded validators
    pub fn last_validators<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<Vec<Validator>, GasStoreErrors> {
        let store = ctx.kv_store(&self.store_key);
        let validators_store = store.prefix_store(LAST_VALIDATOR_POWER_KEY);

        // add the actual validator power sorted store
        let max_validators = self.staking_params_keeper.try_get(ctx)?.max_validators() as usize;
        let mut validators = Vec::with_capacity(max_validators);
        for (i, next) in validators_store.into_range(..).enumerate() {
            let (k, _v) = next?;
            assert!(
                i < max_validators,
                "more validators than maxValidators found"
            );
            let last_validator = ValAddress::try_from_prefix_length_bytes(&k).unwrap_or_corrupt();
            let validator = self
                .validator(ctx, &last_validator)?
                .expect("validator stored in last validators queue should be present in store");
            validators.push(validator);
        }
        Ok(validators)
    }

    pub fn set_last_validator_power<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &LastValidatorPower,
    ) -> Result<(), GasStoreErrors> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut store = store.prefix_store_mut(LAST_VALIDATOR_POWER_KEY);
        let key = validator.address.prefix_len_bytes();
        let value = i64::encode_to_vec(&validator.power);
        store.set(key, value)
    }

    pub fn delete_last_validator_power<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &ValAddress,
    ) -> Result<Option<Vec<u8>>, GasStoreErrors> {
        let store = ctx.kv_store_mut(&self.store_key);
        let mut store = store.prefix_store_mut(LAST_VALIDATOR_POWER_KEY);
        store.delete(&validator.prefix_len_bytes())
    }
}
