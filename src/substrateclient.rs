use crate::traits::{EvmDeposit, ObEvmDeposit, TheaMessage};
use ethers::utils::{hex, keccak256};
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sp_core::hashing::sha2_256;
use subxt::config::SubstrateConfig;
use subxt::dynamic::Value;
use subxt::utils::{AccountId32, H256};
use subxt::{Config, OnlineClient, PolkadotConfig};
use subxt_signer::ecdsa::{Keypair, Seed, Signature};
use subxt::config::polkadot::PolkadotExtrinsicParamsBuilder as Params;
use subxt_signer::bip39::Mnemonic;
use subxt_signer::sr25519::dev;
use crate::traits::{EthereumOP, EtherumAction};
use thea_primitives::types::SignedMessage;
use tokio::sync::mpsc::UnboundedSender;
use crate::error::RelayerError;

#[subxt::subxt(runtime_metadata_path = "src/metadata.scale")]
pub mod polkadex {}

#[derive(Clone, Debug)]
pub struct SubstrateClient {
    client: OnlineClient<SubstrateConfig>,
    signer: Keypair,
}

impl SubstrateClient {
    pub async fn initialize(url: String) -> Result<Self, RelayerError> {
        let api = OnlineClient::<SubstrateConfig>::from_url(url).await?;
        let seed: Seed = Seed::from(H256::from_low_u64_be(10));
        let signer = subxt_signer::ecdsa::Keypair::from_seed(seed)?;
        let public_key = signer.public_key();
        println!("Public Key {:?}", hex::encode(public_key));
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
    ) -> Result<(), RelayerError> {
        println!("Handling Deposit Event");
        let network_id = 255u8; //TODO: Config network Id
        let incoming_nonce_query =
            subxt::dynamic::storage("Thea", "IncomingNonce", vec![network_id]);
        let incoming_nonce = if let Ok(incoming_nonce) = self
            .client
            .storage()
            .at_latest()
            .await?
            .fetch(&incoming_nonce_query)
            .await?
            .ok_or(RelayerError::UnableToFetchIncomingNonce) {
            let incoming_nonce: u64 = Decode::decode(&mut &incoming_nonce.into_encoded()[..])?;
            incoming_nonce
        } else {
            0
        };
        println!("Incoming Nonce {:?}", incoming_nonce);
        let recipient_add: [u8; 32] = deposit.recipient.try_into().map_err(|_| RelayerError::FailedToConvertAddress)?;
        let recipient_add: AccountId32 = AccountId32::from(recipient_add);
        let deposit = thea_primitives::types::Deposit {
            id: Default::default(), //TODO: Get unq id from event and put it here
            recipient: recipient_add,
            asset_id: deposit.asset_id,
            amount: deposit.amount,
            extra: Default::default(),
        };
        let deposit_vec = vec![deposit];
        let message = polkadex::runtime_types::thea_primitives::types::Message {
            block_no: 0,
            nonce: incoming_nonce.saturating_add(1),
            network: network_id,
            data: deposit_vec.encode(),
            payload_type: polkadex::runtime_types::thea_primitives::types::PayloadType::L1Deposit,
        };
        let thea_deposit_tx = polkadex::tx().thea().submit_incoming_message(message, 1_100_000_000_000u128);
        let from = dev::alice(); //TODO: Change it to Signer
        let phrase = "lens secret trick castle amused fresh panel door table merit dance element";
        let mnemonic = Mnemonic::parse(phrase).unwrap();
        let from = subxt_signer::sr25519::Keypair::from_phrase(&mnemonic, None).unwrap();
        let latest_block = self.client.blocks().at_latest().await?;
        let tx_params = Params::new()
            .tip(1_000)
            .mortal(latest_block.header(), 32)
            .build();
        let result = self
            .client
            .tx()
            .sign_and_submit(&thea_deposit_tx, &from, tx_params)
            .await?;
        println!("Deposit Transaction {:?}", result);
        Ok(())
    }

    pub async fn subscribe_substrate_event_stream(
        &self,
        sender: UnboundedSender<TheaMessage>,
    ) -> Result<(), RelayerError> {
        let network_id = 255u8; //TODO: Config network Id
        // Fetch Outgoing nonce
        //OutgoingNonce
        println!("Subscribing to Withdrawal Events");
        let mut blocks_sub = self.client.blocks().subscribe_finalized().await?;
        let mut processed_finalised_outgoing_nonce: u64 = 32; //TODO: Should read from local storage
        while let Some(block) = blocks_sub.next().await {
            let block = block?;
            let block_hash = block.hash();
            let storage_query = subxt::dynamic::storage("Thea", "SignedOutgoingNonce", vec![network_id]);
            let latest_signed_outgoing_nonce = self.client.storage().at_latest().await?.fetch(&storage_query).await?.unwrap(); //FIXME: Remove unwrap
            let latest_signed_outgoing_nonce: u64 = Decode::decode(&mut &latest_signed_outgoing_nonce.into_encoded()[..])?;
            if latest_signed_outgoing_nonce > processed_finalised_outgoing_nonce {
                let storage_query = polkadex::storage().thea().signed_outgoing_messages(network_id, latest_signed_outgoing_nonce);
                if let Some(result) = self.client
                    .storage()
                    .at_latest()
                    .await?
                    .fetch(&storage_query)
                    .await? {
                    println!("Message found {:?}", result);
                    let message: SignedMessage<sp_core::ecdsa::Signature> = Decode::decode(&mut &result.encode()[..])?;
                    //Convert BTreeMap to Vec<(a,b)>
                    let signatures: Vec<(u32, sp_core::ecdsa::Signature)> = message.signatures.into_iter().map(|(a,b)| (a,b)).collect();
                    println!("Message {:?}", hex::encode(message.message.encode().clone()));
                    // Send message using channel
                    sender.send(TheaMessage::SubstrateMessageWithProof(message.message.encode(), message.validator_set_id,signatures))?;
                    processed_finalised_outgoing_nonce = latest_signed_outgoing_nonce; //TODO: Also update the local storage
                }
            }
        }
        Ok(())
    }

    pub async fn handle_ob_deposit(
        &self,
        deposit: ObEvmDeposit,
    ) -> Result<(), RelayerError> {
        let network_id = 2u8; //TODO: Config network Id
        let incoming_nonce_query =
            subxt::dynamic::storage("Thea", "IncomingNonce", vec![network_id]);
        let incoming_nonce = self
            .client
            .storage()
            .at_latest()
            .await?
            .fetch(&incoming_nonce_query)
            .await?
            .ok_or(RelayerError::UnableToFetchIncomingNonce)?
            .into_encoded();
        let incoming_nonce: u64 = Decode::decode(&mut &incoming_nonce[..])?;
        let deposit = thea_primitives::types::Deposit {
            id: Default::default(),
            recipient: deposit.main_account,
            asset_id: deposit.asset_id,
            amount: deposit.amount,
            extra: Default::default(),
        };
        let deposit_vec = vec![deposit];
        let message = polkadex::runtime_types::thea_primitives::types::Message {
            block_no: 0,
            nonce: incoming_nonce,
            data: deposit_vec.encode(),
            network: 1,
            payload_type: polkadex::runtime_types::thea_primitives::types::PayloadType::L1Deposit,
        };
        let thea_deposit_tx = polkadex::tx().thea().submit_incoming_message(message, 1u128);
        println!("Submitting Deposit to Substrate {:?}", thea_deposit_tx);
        Ok(())
    }
}
