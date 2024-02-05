use std::str::FromStr;

use subxt::{
    backend::{legacy::LegacyRpcMethods, rpc},
    error::{RpcError, TransactionError},
    events::EventDetails,
    tx::{Signer, TxInBlock, TxPayload, TxStatus},
    utils::AccountId32,
    OnlineClient, PolkadotConfig,
};
use subxt_signer::{bip39, sr25519::Keypair};

const TESTNET_RPC_URL: &str = "wss://wsspc1-qa.agung.peaq.network";

pub type Error = Box<dyn std::error::Error>;

pub struct Client {
    api: OnlineClient<PolkadotConfig>,
    rpc_legacy: LegacyRpcMethods<PolkadotConfig>,
    keypair: Keypair,
    peaq_did_api: peaq_gen::api::peaq_did::calls::TransactionApi,
}

impl Client {
    pub async fn new(seed: &str) -> Result<Self, Error> {
        let api = OnlineClient::<PolkadotConfig>::from_url(TESTNET_RPC_URL).await?;
        let rpc = rpc::RpcClient::from_url(TESTNET_RPC_URL).await?;
        let rpc_legacy: LegacyRpcMethods<PolkadotConfig> = LegacyRpcMethods::new(rpc.clone());
        let keypair = Keypair::from_phrase(&bip39::Mnemonic::from_str(seed)?, None)?;
        Ok(Self {
            api,
            rpc_legacy,
            keypair,
            peaq_did_api: peaq_gen::api::peaq_did::calls::TransactionApi {},
        })
    }

    pub async fn add_attribute(&self, name: &str, value: Vec<u8>) -> Result<(), Error> {
        let call =
            self.peaq_did_api.add_attribute(self.address(), name.as_bytes().to_vec(), value, None);
        self.submit_tx(&call, &self.keypair).await?;
        Ok(())
    }

    pub async fn read_attribute<T, F>(
        &self,
        name: &str,
        filter: Option<F>,
    ) -> Result<Option<T>, Error>
    where
        F: Fn(EventDetails<PolkadotConfig>) -> Option<T>,
    {
        let call = self.peaq_did_api.read_attribute(self.address(), name.as_bytes().to_vec());
        let tx = self.submit_tx(&call, &self.keypair).await?;
        self.process_events(tx, filter).await
    }

    async fn submit_tx<Call: TxPayload, S: Signer<PolkadotConfig>>(
        &self,
        call: &Call,
        signer: &S,
    ) -> Result<TxInBlock<PolkadotConfig, OnlineClient<PolkadotConfig>>, Error> {
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
                    return Ok(tx_in_block);
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
        let account_nonce =
            self.api.blocks().at(best_block).await?.account_nonce(account_id).await?;
        Ok(account_nonce)
    }

    async fn process_events<T, F>(
        &self,
        tx: TxInBlock<PolkadotConfig, OnlineClient<PolkadotConfig>>,
        filter: Option<F>,
    ) -> Result<Option<T>, Error>
    where
        F: Fn(EventDetails<PolkadotConfig>) -> Option<T>,
    {
        if filter.is_none() {
            return Ok(None);
        }
        let filter = filter.unwrap();
        let events = tx.wait_for_success().await?;
        for event in events.iter() {
            let event = event?;
            if let Some(data) = filter(event) {
                return Ok(Some(data));
            }
        }
        Ok(None)
    }

    fn address(&self) -> AccountId32 {
        <subxt_signer::sr25519::Keypair as Signer<PolkadotConfig>>::account_id(&self.keypair)
    }
}
