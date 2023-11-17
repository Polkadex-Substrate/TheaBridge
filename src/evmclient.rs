use crate::traits::{EvmDeposit, ObEvmDeposit, TheaMessage, VerificationMode};
use ethers::abi::{Address, Contract, Token};
use ethers::contract::Contract as ContractType;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::{LocalWallet, Middleware, Signer, TransactionRequest};
use ethers::providers::Ws;
use ethers::{
    contract::abigen,
    core::types::ValueOrArray,
    providers::{Provider, StreamExt},
};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use vrf::openssl::CipherSuite;
use vrf::openssl::ECVRF;
use vrf::VRF;

abigen!(
    AggregatorInterface,
    r#"[
        event DepositEvent(bytes recipient, uint128 assetId, uint256 amount)
    ]"#,
);

abigen!(
    AggregatorInterfaceOb,
    r#"[
        event DepositEventOb(bytes mainAccount, bytes tradingAccount, uint128 assetId, uint256 amount)
    ]"#,
);

#[derive(Clone, Debug)]
pub struct EvmClient {
    url: String,
    provider: Provider<Ws>,
    contract: Contract,
    wallet: LocalWallet,
    contract_address: Address,
    verification_mode: VerificationMode,
    vrf_secret_key: Vec<u8>,
}

impl EvmClient {
    pub async fn new(
        url: String,
        contract: Contract,
        seed: String,
        contract_address: String,
        verification_mode: VerificationMode,
        vrf_seed: String,
    ) -> Self {
        let provider = Provider::<Ws>::connect(
            "wss://sepolia.infura.io/ws/v3/93554318ae184575adc64c64e2aa7e0c",
        )
        .await
        .unwrap();
        let wallet: LocalWallet = seed.as_str().parse().unwrap();
        Self {
            url,
            provider,
            contract,
            wallet,
            contract_address: contract_address.parse().unwrap(),
            verification_mode,
            vrf_secret_key: hex::decode(vrf_seed).unwrap(),
        }
    }

    /// Returns a set of random bytes to be used and proof
    pub fn generate_vrf_randomness(&self, message: &Vec<u8>) -> (Vec<u8>, Vec<u8>) {
        let mut vrf = ECVRF::from_suite(CipherSuite::SECP256K1_SHA256_TAI).unwrap();
        let pi = vrf.prove(&self.vrf_secret_key, message).unwrap();
        let hash = vrf.proof_to_hash(&pi).unwrap();
        (hash, pi)
    }

    pub async fn subscribe_deposit_events_stream(
        &self,
        sender: UnboundedSender<TheaMessage>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Subscribing to deposit events");
        let event =
            ContractType::event_of_type::<DepositEventFilter>(Arc::new(self.provider.clone()))
                .address(ValueOrArray::Array(vec![
                    self.contract_address, //TODO: Make it part of config
                ]));
        let mut stream = event.subscribe_with_meta().await?.take(2);
        log::info!("Subscribed to deposit events");
        while let Some(Ok((event, meta))) = stream.next().await {
            log::info!("Deposit event received");
            let deposit = EvmDeposit::new(
                event.recipient.clone().to_vec(),
                event.asset_id.clone(),
                event.amount.clone().as_u128(),
            );
            sender.send(TheaMessage::EvmDeposit(deposit)).unwrap();
        }
        Ok(())
    }

    pub async fn subscribe_ob_deposit_events_stream(
        &self,
        sender: UnboundedSender<TheaMessage>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event =
            ContractType::event_of_type::<DepositEventObFilter>(Arc::new(self.provider.clone()))
                .address(ValueOrArray::Array(vec![
                    "0xFF80d05afCb043b5714Bb9a76fD94efB0844F266".parse()?, //TODO: Make it part of config
                ]));
        let mut stream = event.subscribe_with_meta().await?.take(2);
        while let Some(Ok((event, meta))) = stream.next().await {
            let deposit = ObEvmDeposit::new(
                event.main_account.clone().to_vec(),
                event.trading_account.clone().to_vec(),
                event.asset_id.clone(),
                event.amount.clone().as_u128(),
            );
            sender.send(TheaMessage::ObEvmDeposit(deposit)).unwrap();
        }
        Ok(())
    }

    pub async fn handle_substrate_message(
        &self,
        message: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.verification_mode {
            VerificationMode::Relayer => {}
            VerificationMode::VRF => {
                let (randomness, proof) = self.generate_vrf_randomness(&message);
                // TODO: @ZK, Use the randomness as you see fit for the smart contract
            }
        }

        let signature = Token::Bytes(
            self.wallet
                .sign_message(message.clone())
                .await
                .unwrap()
                .to_vec(),
        );
        let message_token = Token::Bytes(message);
        let token_array = vec![message_token, signature];
        let data = self
            .contract
            .function("sendMessage")?
            .encode_input(&token_array)?;
        let tx = TransactionRequest::new();
        let tx = tx.to(self.contract_address).data(data).chain_id(3);
        let mut client = SignerMiddleware::new(self.provider.clone(), self.wallet.clone());
        if let Ok(pending_tx) = client.send_transaction(tx, None).await {
            log::info!("Transaction sent: {:?}", pending_tx);
        }
        Ok(())
    }
}
