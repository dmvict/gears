use cosmwasm_std::Uint256;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tendermint::types::proto::Protobuf;

use crate::types::{base::errors::CoinError, denom::Denom, errors::Error};

use super::Coin;

mod inner {
    pub use core_types::base::coin::Coin;
    pub use core_types::base::coin::IntProto;
}

/// Coin defines a token with a denomination and an amount.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(try_from = "inner::Coin", into = "inner::Coin")]
pub struct UnsignedCoin {
    pub denom: Denom,
    pub amount: Uint256,
}

impl Coin<Uint256> for UnsignedCoin {
    fn denom(&self) -> &Denom {
        &self.denom
    }

    fn amount(&self) -> &Uint256 {
        &self.amount
    }
}

impl TryFrom<inner::Coin> for UnsignedCoin {
    type Error = CoinError;

    fn try_from(value: inner::Coin) -> Result<Self, Self::Error> {
        let denom = value
            .denom
            .try_into()
            .map_err(|e: Error| CoinError::Denom(e.to_string()))?;
        let amount =
            Uint256::from_str(&value.amount).map_err(|e| CoinError::Uint(e.to_string()))?;

        Ok(UnsignedCoin { denom, amount })
    }
}

impl From<UnsignedCoin> for inner::Coin {
    fn from(value: UnsignedCoin) -> inner::Coin {
        Self {
            denom: value.denom.to_string(),
            amount: value.amount.to_string(),
        }
    }
}

impl Protobuf<inner::Coin> for UnsignedCoin {}

impl FromStr for UnsignedCoin {
    type Err = CoinError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // get the index at which amount ends and denom starts
        let i = input.find(|c: char| !c.is_numeric()).unwrap_or(input.len());

        let amount = input[..i]
            .parse::<Uint256>()
            .map_err(|e| CoinError::Uint(e.to_string()))?;

        let denom = input[i..]
            .parse::<Denom>()
            .map_err(|e| CoinError::Denom(e.to_string()))?;

        Ok(UnsignedCoin { denom, amount })
    }
}

/// Uint256Proto is a proto wrapper around Uint256 to allow for proto serialization.
#[derive(Clone, Serialize, Deserialize)]
pub struct Uint256Proto {
    pub uint: Uint256,
}

impl TryFrom<inner::IntProto> for Uint256Proto {
    type Error = CoinError;

    fn try_from(value: inner::IntProto) -> Result<Self, Self::Error> {
        let uint = Uint256::from_str(&value.int).map_err(|e| CoinError::Uint(e.to_string()))?;
        Ok(Uint256Proto { uint })
    }
}

impl From<Uint256Proto> for inner::IntProto {
    fn from(value: Uint256Proto) -> inner::IntProto {
        Self {
            int: value.uint.to_string(),
        }
    }
}

impl Protobuf<inner::IntProto> for Uint256Proto {}
