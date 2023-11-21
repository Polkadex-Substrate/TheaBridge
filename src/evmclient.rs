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
use tokio::sync::mpsc::UnboundedSender;

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
}

impl EvmClient {
    pub async fn new(
        url: String,
        contract: Contract,
        seed: String,
        contract_address: String,
    ) -> Self {
        let provider = Provider::<Ws>::connect(url.clone()).await.unwrap();
        let wallet: LocalWallet = seed.as_str().parse().unwrap();
        let wallet = wallet.with_chain_id(11155111u64);
        Self {
            url,
            provider,
            contract,
            wallet,
            contract_address: contract_address.parse().unwrap(),
        }
    }

    pub async fn subscribe_deposit_events_stream(
        &self,
        sender: UnboundedSender<TheaMessage>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event =
            ContractType::event_of_type::<DepositEventFilter>(Arc::new(self.provider.clone()))
                .address(ValueOrArray::Array(vec![
                    self.contract_address, //TODO: Make it part of config
                ]));
        let mut stream = event.subscribe_with_meta().await?.take(2);
        while let Some(Ok((event, meta))) = stream.next().await {
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
        let mut signature = self
            .wallet
            .sign_hash(H256::from(keccak256(message.clone())))
            .unwrap();
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
        let pending_tx = client.send_transaction(tx, None).await.unwrap();
        Ok(())
    }
}
