use crate::traits::{EvmDeposit, ObEvmDeposit, TheaMessage};
use parity_scale_codec::{Decode, Encode};
use subxt::config::SubstrateConfig;
use subxt::utils::H256;
use subxt::OnlineClient;
use subxt_signer::ecdsa::{Keypair, Seed, Signature};
use thea_primitives::ethereum::{EthereumOP, EtherumAction};
use tokio::sync::mpsc::UnboundedSender;

#[subxt::subxt(runtime_metadata_path = "src/metadata.scale")]
pub mod polkadex {}

#[derive(Clone, Debug)]
pub struct SubstrateClient {
    client: OnlineClient<SubstrateConfig>,
    signer: Keypair,
}

impl SubstrateClient {
    pub async fn initialize(url: String) -> Result<Self, Box<dyn std::error::Error>> {
        let api = OnlineClient::<SubstrateConfig>::from_url(url).await?;
        let seed: Seed = Seed::from(H256::from_low_u64_be(10));
        let signer = subxt_signer::ecdsa::Keypair::from_seed(seed).unwrap();
        let update_task = api.updater();
        tokio::spawn(async move {
            update_task
                .perform_runtime_updates()
                .await
                .expect("Expected the upgrade to work fine");
        });
        Ok(Self {
            client: api,
            signer,
        })
    }

    pub async fn handle_deposit(
        &self,
        deposit: EvmDeposit,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let network_id = 1u8; //TODO: Config network Id
        let incoming_nonce_query =
            subxt::dynamic::storage("Thea", "IncomingNonce", vec![network_id]);
        let incoming_nonce = self
            .client
            .storage()
            .at_latest()
            .await?
            .fetch(&incoming_nonce_query)
            .await?
            .unwrap()
            .into_encoded();
        let incoming_nonce: u64 = Decode::decode(&mut &incoming_nonce[..]).unwrap();
        let deposit = EtherumAction::Deposit(deposit.asset_id, deposit.amount, deposit.recipient);
        let evm_op = EthereumOP {
            txn_id: Default::default(),
            action: deposit,
        };
        let message = polkadex::runtime_types::thea_primitives::types::Message {
            block_no: 0,
            nonce: incoming_nonce,
            data: evm_op.encode(),
            network: 0,
            is_key_change: false,
            validator_set_id: 0,
        };
        let signature: Signature = self.signer.sign(&mut &message.encode());
        let signature = sp_core::ecdsa::Signature(signature.0);
        //let signature: polkadex::runtime_types::thea::ecdsa::app_ecdsa::Signature = polkadex::runtime_types::thea::ecdsa::app_ecdsa::Signature(polkadex::runtime_types::sp_core::ecdsa::Signature(signature.0));
        let signature = Decode::decode(&mut &signature.encode()[..]).unwrap();
        let thea_deposit_tx = polkadex::tx()
            .thea()
            .incoming_message(message, vec![(1u16, signature)]);
        Ok(())
    }

    pub async fn subscribe_substrate_event_stream(
        &self,
        sender: UnboundedSender<TheaMessage>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let keys = Vec::<()>::new();
        let storage_query = subxt::dynamic::storage("Thea", "OutgoingMessages", keys);
        let mut results = self
            .client
            .storage()
            .at_latest()
            .await?
            .iter(storage_query)
            .await?;
        while let Some(Ok((_, value))) = results.next().await {
            sender
                .send(TheaMessage::SubstrateMessage(value.into_encoded()))
                .unwrap();
        }
        Ok(())
    }

    pub async fn handle_ob_deposit(
        &self,
        deposit: ObEvmDeposit,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let network_id = 1u8; //TODO: Config network Id
        let incoming_nonce_query =
            subxt::dynamic::storage("Thea", "IncomingNonce", vec![network_id]);
        let incoming_nonce = self
            .client
            .storage()
            .at_latest()
            .await?
            .fetch(&incoming_nonce_query)
            .await?
            .unwrap()
            .into_encoded();
        let incoming_nonce: u64 = Decode::decode(&mut &incoming_nonce[..]).unwrap();
        let deposit = EtherumAction::DepositToOrderbook(
            deposit.asset_id,
            deposit.amount,
            deposit.main_account,
            deposit.trading_account,
        );
        let evm_op = EthereumOP {
            txn_id: Default::default(),
            action: deposit,
        };
        let message = polkadex::runtime_types::thea_primitives::types::Message {
            block_no: 0,
            nonce: incoming_nonce,
            data: evm_op.encode(),
            network: 0,
            is_key_change: false,
            validator_set_id: 0,
        };
        let signature: Signature = self.signer.sign(&mut &message.encode());
        let signature = sp_core::ecdsa::Signature(signature.0);
        let signature = Decode::decode(&mut &signature.encode()[..]).unwrap();
        let thea_deposit_tx = polkadex::tx()
            .thea()
            .incoming_message(message, vec![(1u16, signature)]);
        Ok(())
    }
}
