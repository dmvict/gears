use bytes::Bytes;
use gears::types::{address::AccAddress, base::send::SendCoins};
use gears::{
    core::{any::google::Any, errors::CoreError},
    error::IBC_ENCODE_UNWRAP,
    tendermint::types::proto::Protobuf,
    types::{base::errors::CoinsError, tx::TxMessage},
};
use serde::{Deserialize, Serialize};

use crate::msg::GovMsg;

mod inner {
    pub use ibc_proto::cosmos::gov::v1beta1::MsgDeposit;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Deposit {
    pub proposal_id: u64,
    pub depositor: AccAddress,
    pub amount: SendCoins,
}

impl Deposit {
    pub(crate) const KEY_PREFIX: [u8; 1] = [0x10];
    pub const TYPE_URL: &'static str = "/cosmos.gov.v1beta1/MsgDeposit";

    pub(crate) fn key(proposal_id: u64, depositor: &AccAddress) -> Vec<u8> {
        [
            Self::KEY_PREFIX.as_slice(),
            &proposal_id.to_be_bytes(),
            &[depositor.len()],
            depositor.as_ref(),
        ]
        .concat()
    }
}

impl Protobuf<inner::MsgDeposit> for Deposit {}

impl TxMessage for Deposit {
    fn get_signers(&self) -> Vec<&AccAddress> {
        vec![&self.depositor]
    }

    fn validate_basic(&self) -> Result<(), String> {
        Ok(())
    }

    fn type_url(&self) -> &'static str {
        Deposit::TYPE_URL
    }
}

impl TryFrom<inner::MsgDeposit> for Deposit {
    type Error = CoreError;

    fn try_from(
        inner::MsgDeposit {
            proposal_id,
            depositor,
            amount,
        }: inner::MsgDeposit,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id,
            depositor: AccAddress::from_bech32(&depositor)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            amount: {
                let mut coins = Vec::with_capacity(amount.len());
                for coin in amount {
                    coins.push(
                        coin.try_into()
                            .map_err(|e: CoinsError| CoreError::Coin(e.to_string()))?,
                    )
                }
                SendCoins::new(coins).map_err(|e| CoreError::DecodeAddress(e.to_string()))?
            },
        })
    }
}

impl From<Deposit> for inner::MsgDeposit {
    fn from(
        Deposit {
            proposal_id,
            depositor,
            amount,
        }: Deposit,
    ) -> Self {
        Self {
            proposal_id,
            depositor: depositor.into(),
            amount: amount
                .into_inner()
                .into_iter()
                .map(|this| this.into())
                .collect(),
        }
    }
}

impl TryFrom<Any> for Deposit {
    type Error = CoreError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        if value.type_url != Self::TYPE_URL {
            Err(CoreError::DecodeGeneral(
                "message type not recognized".into(),
            ))?
        }
        Deposit::decode::<Bytes>(value.value.into())
            .map_err(|e| CoreError::DecodeProtobuf(e.to_string()))
    }
}

impl From<Deposit> for Any {
    fn from(msg: Deposit) -> Self {
        Any {
            type_url: Deposit::TYPE_URL.to_string(),
            value: msg.encode_vec().expect(IBC_ENCODE_UNWRAP),
        }
    }
}

impl From<Deposit> for GovMsg {
    fn from(value: Deposit) -> Self {
        Self::Deposit(value)
    }
}
