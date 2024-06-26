use anyhow::anyhow;
use core_types::{
    signing::SignDoc,
    tx::mode_info::{ModeInfo, SignMode},
};
use prost::Message;
use tendermint::types::{chain_id::ChainId, proto::Protobuf};

use crate::{
    application::handlers::client::MetadataViaRPC,
    error::IBC_ENCODE_UNWRAP,
    signing::{handler::SignModeHandler, renderer::value_renderer::ValueRenderer},
    types::{
        auth::{fee::Fee, info::AuthInfo, tip::Tip},
        signing::SignerInfo,
        tx::{body::TxBody, data::TxData, raw::TxRaw, signer::SignerData, TxMessage},
    },
};

use super::keys::{GearsPublicKey, ReadAccAddress, SigningKey};

/// Contains info required to sign a Tx
pub struct SigningInfo<K> {
    pub key: K,
    pub sequence: u64,
    pub account_number: u64,
}

#[derive(Clone)]
pub enum Mode {
    Direct,
    Textual,
}

impl From<Mode> for SignMode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Direct => SignMode::Direct,
            Mode::Textual => SignMode::Textual,
        }
    }
}

pub fn create_signed_transaction<
    M: TxMessage + ValueRenderer,
    K: SigningKey + ReadAccAddress + GearsPublicKey,
>(
    signing_infos: Vec<SigningInfo<K>>,
    tx_body: TxBody<M>,
    fee: Fee,
    tip: Option<Tip>,
    chain_id: ChainId,
    mode: Mode,
    node: url::Url,
) -> anyhow::Result<TxRaw> {
    let signer_infos: Vec<SignerInfo> = signing_infos
        .iter()
        .map(|s| {
            let public_key = Some(s.key.get_gears_public_key());

            SignerInfo {
                public_key,
                mode_info: ModeInfo::Single(mode.clone().into()),
                sequence: s.sequence,
            }
        })
        .collect();

    let auth_info = AuthInfo {
        signer_infos,
        fee,
        tip,
    };

    let body_bytes = tx_body.encode_vec().expect(IBC_ENCODE_UNWRAP); // TODO:IBC
    let auth_info_bytes = auth_info.encode_vec().expect(IBC_ENCODE_UNWRAP); // TODO:IBC

    let signatures = match mode {
        Mode::Direct => {
            let mut sign_doc = SignDoc {
                body_bytes: body_bytes.clone(),
                auth_info_bytes: auth_info_bytes.clone(),
                chain_id: chain_id.into(),
                account_number: 0, // This gets overwritten
            };

            signing_infos
                .iter()
                .map(|s| {
                    sign_doc.account_number = s.account_number;

                    s.key.sign(&sign_doc.encode_to_vec())
                })
                .collect::<Result<Vec<Vec<u8>>, <K as crate::crypto::keys::SigningKey>::Error>>()
                .map_err(|e| anyhow!(e.to_string()))?
        }
        Mode::Textual => {
            let sign_mode_handler = SignModeHandler;

            signing_infos
                .into_iter()
                .map(|s| {
                    let signer_data = SignerData {
                        address: s.key.get_address(),
                        chain_id: chain_id.clone(),
                        account_number: s.account_number,
                        sequence: s.sequence,
                        pub_key: s.key.get_gears_public_key(),
                    };

                    let tx_data = TxData {
                        body: tx_body.clone(),
                        auth_info: auth_info.clone(),
                    };

                    let sign_bytes = sign_mode_handler
                        .sign_bytes_get(
                            &MetadataViaRPC { node: node.clone() },
                            signer_data,
                            tx_data,
                        )
                        .map_err(|e| anyhow!(e.to_string()))?;

                    s.key.sign(&sign_bytes).map_err(|e| anyhow!(e.to_string()))
                })
                .collect::<Result<Vec<Vec<u8>>, anyhow::Error>>()?
        }
    };

    Ok(TxRaw {
        body_bytes,
        auth_info_bytes,
        signatures,
    })
}
