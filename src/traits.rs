use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
//Import encode
use parity_scale_codec::Encode;

pub trait Message {
    type UnprocessedTheaMessage;
    type ProcessedTheaMessage;
    fn generate_processed_message(
        &self,
        message: Self::UnprocessedTheaMessage,
    ) -> Self::ProcessedTheaMessage;
}

#[derive(Encode)]
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

pub enum TheaMessage {
    EvmDeposit(EvmDeposit),
    ObEvmDeposit(ObEvmDeposit),
    SubstrateMessage(Vec<u8>),
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

#[derive(Debug, Clone)]
pub enum VerificationMode {
    Relayer,
    VRF,
}

// any error type implementing Display is acceptable.
type ParseError = &'static str;

impl std::str::FromStr for VerificationMode {
    type Err = ParseError;
    fn from_str(mode: &str) -> Result<Self, Self::Err> {
        match mode {
            "relayer" => Ok(VerificationMode::Relayer),
            "vrf" => Ok(VerificationMode::VRF),
            _ => Err("Could not parse the verification mode"),
        }
    }
}
