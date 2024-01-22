use parity_scale_codec::Encode;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use parity_scale_codec::Decode;
use scale_info::TypeInfo;
use sp_core::H256;

pub trait Message {
    type UnprocessedTheaMessage;
    type ProcessedTheaMessage;
    fn generate_processed_message(
        &self,
        message: Self::UnprocessedTheaMessage,
    ) -> Self::ProcessedTheaMessage;
}

#[derive(Encode, Debug, Serialize, Deserialize)]
pub struct EvmDeposit {
    pub(crate) recipient: Vec<u8>,
    pub(crate) asset_id: u128,
    pub(crate) amount: u128,
}

impl EvmDeposit {
    pub fn new(recipient: Vec<u8>, asset_id: u128, amount: u128) -> Self {
        Self {
            recipient,
            asset_id,
            amount,
        }
    }
}

#[derive(Encode, Debug, Serialize, Deserialize)]
pub struct ObEvmDeposit {
    pub main_account: Vec<u8>,
    pub trading_account: Vec<u8>,
    pub asset_id: u128,
    pub amount: u128,
}

impl ObEvmDeposit {
    pub fn new(
        main_account: Vec<u8>,
        trading_account: Vec<u8>,
        asset_id: u128,
        amount: u128,
    ) -> Self {
        Self {
            main_account,
            trading_account,
            asset_id,
            amount,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TheaMessage {
    EvmDeposit(EvmDeposit),
    ObEvmDeposit(ObEvmDeposit),
    SubstrateMessage(Vec<u8>),
    SubstrateMessageWithProof(Vec<u8>, Vec<(u32, sp_core::ecdsa::Signature)>),
}

pub struct Channel<T> {
    sender: UnboundedSender<T>,
    pub receiver: UnboundedReceiver<T>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded_channel::<T>();
        Channel { sender, receiver }
    }

    pub fn sender(&self) -> UnboundedSender<T> {
        self.sender.clone()
    }
}

#[derive(
Clone, Encode, Decode, TypeInfo, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize,
)]
pub enum EtherumAction<AccountId> {
    /// Asset id, Amount, user address
    Deposit(u128, u128, AccountId),
    /// Asset id, Amount, user address, proxy address
    DepositToOrderbook(u128, u128, AccountId, AccountId),
    /// Swap
    Swap,
}

#[derive(
Clone, Encode, Decode, TypeInfo, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize,
)]
pub struct EthereumOP<AccountId> {
    pub txn_id: H256,
    pub action: EtherumAction<AccountId>,
}