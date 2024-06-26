use serde::{Deserialize, Serialize};
use tendermint::types::proto::Protobuf;

use crate::types::{account::Account, address::AccAddress};

mod inner {
    pub use core_types::query::request::account::QueryAccountRequest;
    pub use core_types::query::response::account::QueryAccountResponse;
}

/// QueryAccountResponse is the response type for the Query/Account RPC method.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryAccountResponse {
    /// account defines the account of the corresponding address.
    pub account: Option<Account>,
}

impl TryFrom<inner::QueryAccountResponse> for QueryAccountResponse {
    type Error = core_types::errors::CoreError;

    fn try_from(raw: inner::QueryAccountResponse) -> Result<Self, Self::Error> {
        let account = raw.account.map(|a| a.try_into()).transpose()?;
        Ok(QueryAccountResponse { account })
    }
}

impl From<QueryAccountResponse> for inner::QueryAccountResponse {
    fn from(query: QueryAccountResponse) -> inner::QueryAccountResponse {
        Self {
            account: query.account.map(Into::into),
        }
    }
}

impl Protobuf<inner::QueryAccountResponse> for QueryAccountResponse {}

/// QueryAccountRequest is the request type for the Query/Account RPC method.
#[derive(Clone, PartialEq, Debug)]
pub struct QueryAccountRequest {
    /// address defines the address to query for.
    pub address: AccAddress,
}

impl TryFrom<inner::QueryAccountRequest> for QueryAccountRequest {
    type Error = core_types::errors::CoreError;

    fn try_from(raw: inner::QueryAccountRequest) -> Result<Self, Self::Error> {
        let address = AccAddress::from_bech32(&raw.address)
            .map_err(|e| Self::Error::DecodeAddress(e.to_string()))?;

        Ok(QueryAccountRequest { address })
    }
}

impl From<QueryAccountRequest> for inner::QueryAccountRequest {
    fn from(query: QueryAccountRequest) -> inner::QueryAccountRequest {
        Self {
            address: query.address.to_string(),
        }
    }
}

impl Protobuf<inner::QueryAccountRequest> for QueryAccountRequest {}
