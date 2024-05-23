use crate::{
    consts::{expect::SERDE_ENCODING_DOMAIN_TYPE, keeper::VALIDATORS_BY_POWER_INDEX_KEY},
    Commission, CommissionRates, CommissionRaw, Description,
};
use chrono::Utc;
use gears::{
    core::errors::Error,
    error::AppError,
    tendermint::types::{
        proto::{crypto::PublicKey, validator::ValidatorUpdate, Protobuf},
        time::Timestamp,
    },
    types::{
        address::{AccAddress, ConsAddress, ValAddress},
        decimal256::Decimal256,
        uint::Uint256,
    },
};
use prost::{Enumeration, Message};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Pool {
    pub not_bonded_tokens: Uint256,
    pub bonded_tokens: Uint256,
}

/// Last validator power, needed for validator set update logic
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LastValidatorPower {
    pub address: ValAddress,
    pub power: i64,
}

/// Delegation represents the bond with tokens held by an account. It is
/// owned by one delegator, and is associated with the voting power of one
/// validator.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Delegation {
    pub delegator_address: AccAddress,
    pub validator_address: ValAddress,
    pub shares: Decimal256,
}

/// Delegation represents the bond with tokens held by an account. It is
/// owned by one delegator, and is associated with the voting power of one
/// validator.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UnbondingDelegation {
    pub delegator_address: AccAddress,
    pub validator_address: ValAddress,
    pub entries: Vec<UnbondingDelegationEntry>,
}

/// UnbondingDelegationEntry - entry to an UnbondingDelegation
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UnbondingDelegationEntry {
    pub creation_height: i64,
    pub completion_time: Timestamp,
    pub initial_balance: Uint256,
    pub balance: Uint256,
}

impl UnbondingDelegationEntry {
    pub fn is_mature(&self, time: chrono::DateTime<Utc>) -> bool {
        let completion_time = chrono::DateTime::from_timestamp(
            self.completion_time.seconds,
            self.completion_time.nanos as u32,
        )
        .expect("Invalid timestamp in unbonding delegation entry. It means that timestamp contains out-of-range number of seconds and/or invalid nanosecond");
        completion_time <= time
    }
}

/// Redelegation contains the list of a particular delegator's
/// redelegating bonds from a particular source validator to a
/// particular destination validator
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Redelegation {
    pub delegator_address: AccAddress,
    pub validator_src_address: ValAddress,
    pub validator_dst_address: ValAddress,
    pub entries: Vec<RedelegationEntry>,
}

/// RedelegationEntry - entry to a Redelegation
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RedelegationEntry {
    pub creation_height: i64,
    pub completion_time: Timestamp,
    pub initial_balance: Uint256,
    pub share_dst: Decimal256,
}

impl RedelegationEntry {
    pub fn is_mature(&self, time: chrono::DateTime<Utc>) -> bool {
        let completion_time = chrono::DateTime::from_timestamp(
            self.completion_time.seconds,
            self.completion_time.nanos as u32,
        )
        .expect("Invalid timestamp in unbonding delegation entry. It means that timestamp contains out-of-range number of seconds and/or invalid nanosecond");
        completion_time <= time
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DvvTriplet {
    pub del_addr: AccAddress,
    pub val_src_addr: ValAddress,
    pub val_dst_addr: ValAddress,
}
impl DvvTriplet {
    pub fn new(del_addr: AccAddress, val_src_addr: ValAddress, val_dst_addr: ValAddress) -> Self {
        Self {
            del_addr,
            val_src_addr,
            val_dst_addr,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DvPair {
    pub val_addr: ValAddress,
    pub del_addr: AccAddress,
}
impl DvPair {
    pub fn new(val_addr: ValAddress, del_addr: AccAddress) -> Self {
        Self { val_addr, del_addr }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Enumeration)]
pub enum BondStatus {
    Unbonded = 0,
    Unbonding = 1,
    Bonded = 2,
}

impl Display for BondStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BondStatus::Unbonded => write!(f, "Unbonded"),
            BondStatus::Unbonding => write!(f, "Unbonding"),
            BondStatus::Bonded => write!(f, "Bonded"),
        }
    }
}

/// Validator defines a validator, together with the total amount of the
/// Validator's bond shares and their exchange rate to coins. Slashing results in
/// a decrease in the exchange rate, allowing correct calculation of future
/// undelegations without iterating over delegators. When coins are delegated to
/// this validator, the validator is credited with a delegation whose number of
/// bond shares is based on the amount of coins delegated divided by the current
/// exchange rate. Voting power can be calculated as total bonded shares
/// multiplied by exchange rate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Validator {
    /// operator_address defines the address of the validator's operator; bech encoded in JSON.
    pub operator_address: ValAddress,
    /// delegator_shares defines total shares issued to a validator's delegators.
    pub delegator_shares: Decimal256,
    /// description defines the description terms for the validator.
    pub description: Description,
    /// consensus_pubkey is the consensus public key of the validator, as a Protobuf Any.
    pub consensus_pubkey: PublicKey,
    /// jailed defined whether the validator has been jailed from bonded status or not.
    pub jailed: bool,
    /// tokens define the delegated tokens (incl. self-delegation).
    pub tokens: Uint256,
    /// unbonding_height defines, if unbonding, the height at which this validator has begun unbonding.
    pub unbonding_height: i64,
    /// unbonding_time defines, if unbonding, the min time for the validator to complete unbonding.
    pub unbonding_time: Timestamp,
    /// commission defines the commission parameters.
    pub commission: Commission,
    pub min_self_delegation: Uint256,
    pub status: BondStatus,
}

impl Validator {
    pub fn new_with_defaults(
        operator_address: ValAddress,
        consensus_pubkey: PublicKey,
        description: Description,
    ) -> Validator {
        Validator {
            operator_address,
            delegator_shares: Decimal256::zero(),
            description,
            consensus_pubkey,
            jailed: false,
            tokens: Uint256::zero(),
            unbonding_height: 0,
            unbonding_time: Timestamp {
                seconds: 0,
                nanos: 0,
            },
            commission: Commission {
                commission_rates: CommissionRates {
                    rate: Decimal256::zero(),
                    max_rate: Decimal256::zero(),
                    max_change_rate: Decimal256::zero(),
                },
                update_time: Timestamp {
                    seconds: 0,
                    nanos: 0,
                },
            },
            min_self_delegation: Uint256::one(),
            status: BondStatus::Unbonded,
        }
    }

    pub fn abci_validator_update(&self, power: i64) -> ValidatorUpdate {
        ValidatorUpdate {
            pub_key: self.consensus_pubkey.clone(),
            power: self.consensus_power(power),
        }
    }
    pub fn abci_validator_update_zero(&self) -> ValidatorUpdate {
        self.abci_validator_update(0)
    }

    pub fn set_initial_commission(&mut self, commission: Commission) -> Result<(), AppError> {
        commission.validate()?;
        self.commission = commission;
        Ok(())
    }

    /// add_tokens_from_del adds tokens to a validator
    pub fn add_tokens_from_del(&mut self, amount: Uint256) -> Decimal256 {
        // calculate the shares to issue
        let issues_shares = if self.delegator_shares.is_zero() {
            // the first delegation to a validator sets the exchange rate to one
            Decimal256::new(amount)
        } else {
            match self.shares_from_tokens(amount) {
                Ok(shares) => shares,
                Err(err) => panic!("{}", err),
            }
        };

        self.tokens += amount;
        self.delegator_shares += issues_shares;
        issues_shares
    }

    fn shares_from_tokens(&self, amount: Uint256) -> anyhow::Result<Decimal256> {
        if self.tokens.is_zero() {
            return Err(AppError::Custom("insufficient shares".into()).into());
        }
        Ok(self
            .delegator_shares
            .checked_mul(Decimal256::new(amount))?
            .checked_div(Decimal256::new(self.tokens))?)
    }

    /// calculate the token worth of provided shares
    pub fn tokens_from_shares(&self, shares: Decimal256) -> anyhow::Result<Decimal256> {
        Ok(shares
            .checked_mul(Decimal256::new(self.tokens))?
            .checked_div(self.delegator_shares)?)
    }

    pub fn invalid_ex_rate(&self) -> bool {
        self.tokens.is_zero() && (self.delegator_shares > Decimal256::zero())
    }

    pub fn cons_addr(&self) -> ConsAddress {
        self.consensus_pubkey.clone().into()
    }

    pub fn update_status(&mut self, status: BondStatus) {
        self.status = status;
    }

    pub fn tendermint_power(&self) -> i64 {
        if self.status == BondStatus::Bonded {
            return self.potential_tendermint_power();
        }
        0
    }

    pub fn potential_tendermint_power(&self) -> i64 {
        let amount = self.tokens / Uint256::from(10u64).pow(6);
        amount
            .to_string()
            .parse::<i64>()
            .expect("Unexpected conversion error")
    }

    pub fn consensus_power(&self, power: i64) -> i64 {
        match self.status {
            BondStatus::Bonded => self.potential_consensus_power(power),
            _ => 0,
        }
    }

    pub fn potential_consensus_power(&self, power: i64) -> i64 {
        self.tokens_to_consensus_power(power)
    }

    pub fn tokens_to_consensus_power(&self, power: i64) -> i64 {
        let amount = self.tokens / Uint256::from(power as u64);
        amount
            .to_string()
            .parse::<i64>()
            .expect("Unexpected conversion error")
    }

    /// GetValidatorsByPowerIndexKey creates the validator by power index.
    /// Power index is the key used in the power-store, and represents the relative
    /// power ranking of the validator.
    /// VALUE: validator operator address ([]byte)
    pub fn key_by_power_index_key(&self, power_reduction: i64) -> Vec<u8> {
        // NOTE the address doesn't need to be stored because counter bytes must always be different
        // NOTE the larger values are of higher value
        let consensus_power = self.tokens_to_consensus_power(power_reduction);
        let consensus_power_bytes = consensus_power.to_ne_bytes();

        let oper_addr_invr = self
            .operator_address
            .to_string()
            .as_bytes()
            .iter()
            .map(|b| 255 ^ b)
            .collect::<Vec<_>>();

        // key is of format prefix || powerbytes || addrLen (1byte) || addrBytes
        let mut key = VALIDATORS_BY_POWER_INDEX_KEY.to_vec();
        key.extend_from_slice(&consensus_power_bytes);
        key.push(oper_addr_invr.len() as u8);
        key.extend_from_slice(&oper_addr_invr);
        key
    }
}

impl TryFrom<ValidatorRaw> for Validator {
    type Error = Error;
    fn try_from(value: ValidatorRaw) -> Result<Self, Self::Error> {
        let status = value.status();
        Ok(Self {
            operator_address: ValAddress::from_bech32(&value.operator_address)
                .map_err(|e| Error::DecodeAddress(e.to_string()))?,
            delegator_shares: Decimal256::from_str(&value.delegator_shares)
                .map_err(|e| Error::DecodeGeneral(e.to_string()))?,
            description: value
                .description
                .expect("Value should exists. It's the proto3 rule to have Option<T> instead of T"),
            consensus_pubkey: serde_json::from_slice(&value.consensus_pubkey)
                .map_err(|e| Error::DecodeGeneral(e.to_string()))?,
            jailed: value.jailed,
            tokens: Uint256::from_str(&value.tokens)
                .map_err(|e| Error::DecodeGeneral(e.to_string()))?,
            unbonding_height: value.unbonding_height,
            unbonding_time: value
                .unbonding_time
                .expect("Value should exists. It's the proto3 rule to have Option<T> instead of T"),
            commission: value
                .commission
                .expect("Value should exists. It's the proto3 rule to have Option<T> instead of T")
                .try_into()?,
            min_self_delegation: Uint256::from_str(&value.min_self_delegation)
                .map_err(|e| Error::DecodeGeneral(e.to_string()))?,
            status,
        })
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct ValidatorRaw {
    #[prost(string)]
    pub operator_address: String,
    #[prost(string)]
    pub delegator_shares: String,
    #[prost(message, optional)]
    pub description: Option<Description>,
    #[prost(bytes)]
    pub consensus_pubkey: Vec<u8>,
    #[prost(bool)]
    pub jailed: bool,
    #[prost(string)]
    pub tokens: String,
    #[prost(int64)]
    pub unbonding_height: i64,
    #[prost(message, optional)]
    pub unbonding_time: Option<Timestamp>,
    #[prost(message, optional)]
    pub commission: Option<CommissionRaw>,
    #[prost(string)]
    pub min_self_delegation: String,
    #[prost(enumeration = "BondStatus")]
    pub status: i32,
}

impl From<Validator> for ValidatorRaw {
    fn from(value: Validator) -> Self {
        Self {
            operator_address: value.operator_address.to_string(),
            delegator_shares: value.delegator_shares.to_string(),
            description: Some(value.description),
            consensus_pubkey: serde_json::to_vec(&value.consensus_pubkey)
                .expect(SERDE_ENCODING_DOMAIN_TYPE),
            jailed: value.jailed,
            tokens: value.tokens.to_string(),
            unbonding_height: value.unbonding_height,
            unbonding_time: Some(value.unbonding_time),
            commission: Some(value.commission.into()),
            min_self_delegation: value.min_self_delegation.to_string(),
            status: value.status.into(),
        }
    }
}

impl Protobuf<ValidatorRaw> for Validator {}
