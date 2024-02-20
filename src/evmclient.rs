use std::str::FromStr;
use crate::traits::{EvmDeposit, Message, ObEvmDeposit, TheaMessage};
use ethers::abi::{Address, Contract, Token};
use ethers::contract::stream::EventStream;
use ethers::contract::Contract as ContractType;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::{Http, LocalWallet, Middleware, Signer, TransactionRequest, H256};
use ethers::providers::Ws;
use ethers::utils::{hex, keccak256};
use ethers::{
    contract::abigen,
    core::types::ValueOrArray,
    providers::{Provider, StreamExt},
};
use std::sync::Arc;
use ethers::types::H160;
use sp_application_crypto::RuntimeAppPublic;
use sp_core::U256;
use thea_primitives::ValidatorSetId;
use tokio::sync::mpsc::UnboundedSender;
use vrf::openssl::{CipherSuite, ECVRF};
use vrf::VRF;
use crate::error::RelayerError;

// abigen!(
//     AggregatorInterface,
//     r#"[
//         event DepositEvent(bytes recipient, uint128 assetId, uint256 amount)
//     ]"#,
// );
//
// abigen!(
//     AggregatorInterfaceOb,
//     r#"[
//         event DepositEventOb(bytes mainAccount, bytes tradingAccount, uint128 assetId, uint256 amount)
//     ]"#,
// );

#[derive(Clone, Debug)]
pub struct EvmClient {
    url: String,
    provider: Provider<Ws>,
    contract: Contract,
    thea_contract: TheaContract<Provider<Ws>>,
    wallet: LocalWallet,
    contract_address: Address
}

abigen!(
    TheaContract,
    "./thea_abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

impl EvmClient {
    pub async fn new(
        url: String,
        contract: Contract,
        seed: String,
        contract_address: String
    ) -> Result<Self, RelayerError> {
        let provider = Provider::<Ws>::connect(url.clone()).await?;
        let wallet: LocalWallet = seed.as_str().parse()?;
        let wallet = wallet.with_chain_id(11155111u64);
        let thea_contract = TheaContract::new(contract_address.parse::<Address>().map_err(|_| RelayerError::HexConversionError)?, provider.clone().into());
        Ok(Self {
            url,
            provider,
            contract,
            wallet,
            thea_contract,
            contract_address: contract_address.parse().map_err(|_| RelayerError::HexConversionError)?
        })
    }

    pub async fn subscribe_deposit_events_stream(
        &self,
        sender: UnboundedSender<TheaMessage>,
    ) -> Result<(), RelayerError> {
        println!("Subscribed deposit events");
        let event =
            ContractType::event_of_type::<DepositEventFilter>(Arc::new(self.provider.clone()))
                .address(ValueOrArray::Array(vec![
                    self.contract_address,
                ]));
        let mut stream = event.subscribe_with_meta().await?.take(2);
        while let Some(Ok((event, _meta))) = stream.next().await {
            println!("Got Deposit Event");
            let deposit = EvmDeposit::new(
                event.recipient.clone().to_vec(),
                event.asset_id.clone(),
                event.amount.clone().as_u128(),
            );
            sender.send(TheaMessage::EvmDeposit(deposit))?;
        }
        Ok(())
    }

    pub async fn subscribe_ob_deposit_events_stream(
        &self,
        sender: UnboundedSender<TheaMessage>,
    ) -> Result<(), RelayerError> {
        let event =
            ContractType::event_of_type::<DepositEventObFilter>(Arc::new(self.provider.clone()))
                .address(ValueOrArray::Array(vec![
                    self.contract_address,
                ]));
        let mut stream = event.subscribe_with_meta().await?.take(2);
        while let Some(Ok((event, _meta))) = stream.next().await {
            let deposit = ObEvmDeposit::new(
                event.main_account.clone().to_vec(),
                event.trading_account.clone().to_vec(),
                event.asset_id.clone(),
                event.amount.clone().as_u128(),
            );
            sender.send(TheaMessage::ObEvmDeposit(deposit))?;
        }
        Ok(())
    }

    pub async fn handle_substrate_message(
        &self,
        message: Vec<u8>,
    ) -> Result<(), RelayerError> {
        let signature = self
            .wallet
            .sign_hash(H256::from(keccak256(message.clone())))?;
        let signature = signature.to_vec();
        let signature = Token::Bytes(signature);
        let message_token = Token::Bytes(message);
        let token_array = vec![message_token, signature];
        let data = self
            .contract
            .function("sendMessage")?
            .encode_input(&token_array)?;
        let tx = TransactionRequest::new();
        let tx = tx.to(self.contract_address).data(data).chain_id(11155111);
        let mut client = SignerMiddleware::new(self.provider.clone(), self.wallet.clone());
        let pending_tx = client.send_transaction(tx, None).await?;
        println!("Pending tx_id {:?}", pending_tx);
        Ok(())
    }

    pub async fn get_validator_index(&self, message: Vec<u8>, validator_set_id: u64, indexes: Vec<u64>) -> Result<Vec<u64>, RelayerError> {
        let indexes: Vec<u64> = self.thea_contract.get_validator_index(message.into(), validator_set_id.into(), indexes).call().await?;
        Ok(indexes)
    }

    pub async fn handle_substrate_message_with_proof(&self, message: Vec<u8>, validator_set_id: ValidatorSetId ,signatures: Vec<(u32, sp_core::ecdsa::Signature)>) -> Result<(), RelayerError> {

        println!("Got Message from Substrate");
        println!("Substrate Message is: {:?}", hex::encode(message.clone()));
        let signature_indexes: Vec<u64> = signatures.iter().map(|(index, _)| *index as u64).collect();
        println!("signatue_index KSR {:?}", signature_indexes.clone());
        let indexes: Vec<u64> = self.get_validator_index(message.clone(), validator_set_id, signature_indexes.clone()).await?;
        println!("Indexes {:?}", indexes.clone());
        let mut final_signatures: Vec<Token> = vec![];
        println!("Raw Sig Len {:?}", signatures.len());
        for i in indexes.clone() {
            for (index, sig) in signatures.clone() {
                if i == index as u64 {
                    let sig = sig.0.to_vec();
                    println!("Indexed Signature {:?}", hex::encode(sig.clone()));
                    final_signatures.push(Token::Bytes(sig));
                }
            }
        }
        println!("Final Signature Len {:?}", final_signatures.len());
        //println!("Final Sig {:?}", final_signatures.clone());
        let signatures = Token::Array(final_signatures);
        let message_token = Token::Bytes(message);
        let signature_indexes_token: Vec<Token> = signature_indexes.iter().map(|index| Token::Uint(U256::from(*index))).collect();
        println!("Signature Indexes Len {:?}", signature_indexes_token.len());
        let siganture_indexes = Token::Array(signature_indexes_token);
        let token_array = vec![message_token, signatures, siganture_indexes];
        let data = self
            .contract
            .function("sendMessage")?
            .encode_input(&token_array)?;
        let tx = TransactionRequest::new();
        let tx = tx.to(self.contract_address).data(data).chain_id(11155111); //TODO: Make it part
        let client = SignerMiddleware::new(self.provider.clone(), self.wallet.clone());
        let pending_tx = client.send_transaction(tx, None).await?;
        println!("pending_tx: {:?}", pending_tx);
        Ok(())
    }
}
