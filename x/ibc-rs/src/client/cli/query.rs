use gears::application::handlers::client::QueryHandler;

use crate::ics02_client::client::cli::query::query_handler::ClientQueryHandler;

use std::borrow::Cow;

use clap::{Args, Subcommand};
use gears::baseapp::Query;
use serde::{Deserialize, Serialize};

use crate::ics02_client::client::cli::query::{ClientQuery, ClientQueryCli, ClientQueryResponse};

/// Querying commands for the ibc module
#[derive(Args, Debug)]
pub struct IbcQueryCli {
    #[command(subcommand)]
    pub command: IbcQueryCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum IbcQueryCommands {
    Client(ClientQueryCli),
}

#[derive(Clone, PartialEq)]
pub enum IbcQuery {
    Client(ClientQuery),
}

impl Query for IbcQuery {
    fn query_url(&self) -> &'static str {
        match self {
            IbcQuery::Client(query) => query.query_url(),
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            IbcQuery::Client(query) => query.into_bytes(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum IbcQueryResponse {
    Client(ClientQueryResponse),
}

#[derive(Debug, Clone)]
pub struct IbcQueryHandler;

impl QueryHandler for IbcQueryHandler {
    type QueryRequest = IbcQuery;
    type QueryCommands = IbcQueryCli;
    type QueryResponse = IbcQueryResponse;

    fn prepare_query_request(
        &self,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryRequest> {
        let res = match &command.command {
            IbcQueryCommands::Client(command) => {
                Self::QueryRequest::Client(ClientQueryHandler.prepare_query_request(command)?)
            }
        };

        Ok(res)
    }

    fn handle_raw_response(
        &self,
        query_bytes: Vec<u8>,
        command: &Self::QueryCommands,
    ) -> anyhow::Result<Self::QueryResponse> {
        let res = match &command.command {
            IbcQueryCommands::Client(command) => Self::QueryResponse::Client(
                ClientQueryHandler.handle_raw_response(query_bytes, command)?,
            ),
        };

        Ok(res)
    }
}
