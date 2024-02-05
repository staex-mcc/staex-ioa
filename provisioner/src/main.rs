use std::str::{from_utf8, FromStr};

use contract_transcode::ContractMessageTranscoder;
use log::debug;
use pallet_contracts_primitives::ContractExecResult;
use subxt::{
    backend::{
        legacy::LegacyRpcMethods,
        rpc::{self, RpcClient},
    },
    error::{RpcError, TransactionError},
    ext::sp_core::H256,
    rpc_params,
    tx::{Signer, TxPayload, TxStatus},
    utils::AccountId32,
    OnlineClient, PolkadotConfig,
};
use subxt_signer::{bip39, sr25519::Keypair};

mod asd;

use crate::asd::api::peaq_did::{
    events::{AttributeAdded, AttributeRead},
    Call,
};
use crate::asd::api::{
    peaq_did::calls::types::add_attribute::Value,
    runtime_types::{frame_system::EventRecord, peaq_dev_runtime::RuntimeEvent},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const RPC_URL: &str = "wss://wsspc1-qa.agung.peaq.network";
    let api = OnlineClient::<PolkadotConfig>::from_url(RPC_URL).await?;
    let rpc = rpc::RpcClient::from_url(RPC_URL).await?;
    let rpc_legacy: LegacyRpcMethods<PolkadotConfig> = LegacyRpcMethods::new(rpc.clone());
    let keypair = Keypair::from_phrase(
        &bip39::Mnemonic::from_str(
            "",
        )?,
        None,
    )?;
    let best_block = rpc_legacy.chain_get_block(None).await?.unwrap();
    eprintln!("{:?}", best_block.block.header.number);

    // let transcoder = ContractMessageTranscoder::load("asd.scale")?;

    let app = App {
        api: api.clone(),
        rpc: rpc.clone(),
        rpc_legacy,
        // transcoder,
        keypair: keypair.clone(),
    };

    let api__ = asd::api::peaq_did::calls::TransactionApi {};
    // let call = api.add_attribute(
    //     "5CS3ZHVZRSKckfQ583aCszSsMiJ6F32kNUGgxTvzdTpdcrCh".parse()?,
    //     "peaq-not-sdk-test".as_bytes().to_vec(),
    //     r#"{"hello": "peaq"}"#.as_bytes().to_vec(),
    //     None,
    // );
    // app.submit_tx(&call, &keypair).await?;

    let call = api__.read_attribute(
        "5CS3ZHVZRSKckfQ583aCszSsMiJ6F32kNUGgxTvzdTpdcrCh".parse()?,
        "peaq-not-sdk-test".as_bytes().to_vec(),
    );
    app.submit_tx(&call, &keypair).await?;

    // let payload = DynamicRuntimeApiPayload::new();
    // let client: RuntimeApiClient<String, OnlineClient<PolkadotConfig>> =
    //     subxt::runtime_api::RuntimeApiClient::new(api)
    //         .at(best_block)
    //         .call(payload)
    //         .await?;

    // let mut i: usize = 1701416;
    // loop {
    //     let res: Result<H256, subxt::Error> =
    //         rpc.request("chain_getBlockHash", rpc_params![i]).await;
    //     if let Err(e) = res {
    //         match e {
    //             subxt::Error::Serialization(_) => return Ok(()),
    //             _ => return Err(e.to_string().into()),
    //         }
    //     }
    //     let hash = res?;
    //     let block = api.blocks().at(hash).await?;
    //     let events = block.events().await?;
    //     for event in events.iter() {
    //         let event = event?;
    //         eprintln!("{} - {:?}", i, event.pallet_index());
    //         eprintln!("{} - {:?}", i, event.pallet_name());
    //         eprintln!("{} - {:?}", i, event.phase());
    //         eprintln!("{} - {:?}", i, event.topics());
    //         eprintln!("{} - {:?}", i, event.variant_name());
    //         // eprintln!("{} - {:?}", i, event.bytes());
    //         if event.variant_name() == "AttributeAdded" {
    //             // let evt = event.as_root_event::<AttributeAdded>();
    //             // eprintln!("{:?}", evt);
    //             if let Ok(evt) = event.as_event::<AttributeAdded>() {
    //                 let evt = evt.unwrap();
    //                 eprintln!("{:?}", from_utf8(&evt.2)); // attribute name
    //                 eprintln!("{:?}", evt.3);
    //                 eprintln!("{:?}", from_utf8(&evt.3));
    //                 eprintln!("{:?}", r#"{"hello": "peaq"}"#.as_bytes());
    //                 // eprintln!("{:?}", from_utf8(&evt.3[..119]));
    //                 // eprintln!("{:?}", scale::decode_from_bytes::<Value>(evt.3.into()));
    //                 // eprintln!("{:?}", "hello---".as_bytes());
    //                 // eprintln!("{:?}", "raw".as_bytes());
    //                 // eprintln!("{:?}", "pizza".as_bytes());
    //             }
    //             return Ok(());
    //         }
    //     }
    //     i += 1;
    //     if i == 1701420 {
    //         break;
    //     }
    // }

    Ok(())
}

type SUBXTConfig = PolkadotConfig;

type Error = Box<dyn std::error::Error>;

#[derive(scale::Encode, scale::Decode, Debug)]
struct Weight {
    #[codec(compact)]
    ref_time: u64,
    #[codec(compact)]
    proof_size: u64,
}

impl Weight {
    fn new(ref_time: u64, proof_size: u64) -> Self {
        Self {
            ref_time,
            proof_size,
        }
    }
}

impl From<Weight> for crate::asd::api::runtime_types::sp_weights::weight_v2::Weight {
    fn from(value: Weight) -> Self {
        Self {
            ref_time: value.ref_time,
            proof_size: value.proof_size,
        }
    }
}

#[derive(Debug)]
struct DryRunResult {
    data: Value,
    gas_required: Weight,
}

struct App {
    api: OnlineClient<SUBXTConfig>,
    rpc: RpcClient,
    rpc_legacy: LegacyRpcMethods<SUBXTConfig>,
    // transcoder: ContractMessageTranscoder,
    keypair: Keypair,
}

impl App {
    async fn submit_tx<Call: TxPayload, S: Signer<SUBXTConfig>>(
        &self,
        call: &Call,
        signer: &S,
    ) -> Result<(), Error> {
        let account_id = signer.account_id();
        let account_nonce = self.get_nonce(&account_id).await?;
        let mut tx = self
            .api
            .tx()
            .create_signed_with_nonce(call, signer, account_nonce, Default::default())?
            .submit_and_watch()
            .await?;
        while let Some(status) = tx.next().await {
            match status? {
                TxStatus::InBestBlock(tx_in_block) | TxStatus::InFinalizedBlock(tx_in_block) => {
                    let events = tx_in_block.wait_for_success().await?;
                    eprintln!("EVENTS LEN: {}", events.all_events_in_block().len());
                    let mut i = 0;
                    for event in events.iter() {
                        i += 1;
                        let event = event?;
                        eprintln!("{} - {:?}", i, event.pallet_index());
                        eprintln!("{} - {:?}", i, event.pallet_name());
                        eprintln!("{} - {:?}", i, event.phase());
                        eprintln!("{} - {:?}", i, event.topics());
                        eprintln!("{} - {:?}", i, event.variant_name());
                        if event.variant_name() == "AttributeRead" {
                            if let Ok(evt) = event.as_event::<AttributeRead>() {
                                let evt = evt.unwrap();
                                eprintln!("{:?}", from_utf8(&evt.0.name)); // attribute name
                                eprintln!("{:?}", from_utf8(&evt.0.value)); // attribute name
                            }
                            return Ok(());
                        }
                    }
                    return Ok(());
                }
                TxStatus::Error { message } => return Err(TransactionError::Error(message).into()),
                TxStatus::Invalid { message } => {
                    return Err(TransactionError::Invalid(message).into())
                }
                TxStatus::Dropped { message } => {
                    return Err(TransactionError::Dropped(message).into())
                }
                _ => continue,
            }
        }
        Err(RpcError::SubscriptionDropped.into())
    }

    async fn get_nonce(&self, account_id: &AccountId32) -> Result<u64, Error> {
        let best_block = self
            .rpc_legacy
            .chain_get_block_hash(None)
            .await?
            .ok_or(subxt::Error::Other("best block not found".into()))?;
        let account_nonce = self
            .api
            .blocks()
            .at(best_block)
            .await?
            .account_nonce(account_id)
            .await?;
        Ok(account_nonce)
    }
}
