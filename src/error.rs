use std::fmt::{Debug, Display, Formatter};
use ethers::prelude::ProviderError;
use ethers::prelude::signer::SignerMiddlewareError;
use ethers::providers::{Provider, Ws};
use ethers::signers::{Wallet, WalletError};
use k256::ecdsa::SigningKey;
use tokio::sync::mpsc::error::SendError;
use crate::traits::TheaMessage;

pub enum RelayerError {
    NativeError,
    UnableToFetchIncomingNonce,
    FailedToConvertAddress,
    CodecError(parity_scale_codec::Error),
    SubxtError(subxt::Error),
    TokioChannelError(tokio::sync::mpsc::error::SendError<TheaMessage>),
    EthersAbiError(ethers::abi::Error),
    EthersContractError(ethers::contract::ContractError<Provider<Ws>>),
    EthersSignerMiddlewareError(SignerMiddlewareError<Provider<Ws>, Wallet<SigningKey>>),
    EthersProviderError(ethers::providers::ProviderError),
    EthersWalletError(WalletError),
    IoError(std::io::Error),
    SubxtSignerError(subxt_signer::ecdsa::Error),
    HexConversionError,
    AuthoritiesNotFound
}

impl Display for RelayerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err_msg = match self {
            RelayerError::NativeError => "Native Error".to_string(),
            RelayerError::UnableToFetchIncomingNonce => "Unable to fetch incoming nonce".to_string(),
            RelayerError::FailedToConvertAddress => "Failed to convert address".to_string(),
            RelayerError::CodecError(error) => format!("Codec Error: {:?}", error),
            RelayerError::SubxtError(error) => format!("Subxt Error: {:?}", error),
            RelayerError::TokioChannelError(error) => format!("Tokio Channel Error: {:?}", error),
            RelayerError::EthersAbiError(error) => format!("Ethers Abi Error: {:?}", error),
            RelayerError::EthersContractError(error) => format!("Ethers Contract Error: {:?}", error),
            RelayerError::EthersSignerMiddlewareError(error) => format!("Ethers Signer Middleware Error: {:?}", error),
            RelayerError::EthersProviderError(error) => format!("Ethers Provider Error: {:?}", error),
            RelayerError::EthersWalletError(error) => format!("Ethers Wallet Error: {:?}", error),
            RelayerError::IoError(error) => format!("Io Error: {:?}", error),
            RelayerError::SubxtSignerError(error) => format!("Subxt Signer Error: {:?}", error),
            RelayerError::HexConversionError => "Hex Conversion Error".to_string(),
            RelayerError::AuthoritiesNotFound => "Authorities not found".to_string(),
        };
        write!(f, "{}", err_msg)
    }
}

impl Debug for RelayerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err_msg = match self {
            RelayerError::NativeError => "Native Error".to_string(),
            RelayerError::UnableToFetchIncomingNonce => "Unable to fetch incoming nonce".to_string(),
            RelayerError::FailedToConvertAddress => "Failed to convert address".to_string(),
            RelayerError::CodecError(error) => format!("Codec Error: {}", error),
            RelayerError::SubxtError(error) => format!("Subxt Error: {}", error),
            RelayerError::TokioChannelError(error) => format!("Tokio Channel Error: {}", error),
            RelayerError::EthersAbiError(error) => format!("Ethers Abi Error: {}", error),
            RelayerError::EthersContractError(error) => format!("Ethers Contract Error: {}", error),
            RelayerError::EthersSignerMiddlewareError(error) => format!("Ethers Signer Middleware Error: {}", error),
            RelayerError::EthersProviderError(error) => format!("Ethers Provider Error: {}", error),
            RelayerError::EthersWalletError(error) => format!("Ethers Wallet Error: {}", error),
            RelayerError::IoError(error) => format!("Io Error: {}", error),
            RelayerError::SubxtSignerError(error) => format!("Subxt Signer Error: {}", error),
            RelayerError::HexConversionError => "Hex Conversion Error".to_string(),
            RelayerError::AuthoritiesNotFound => "Authorities not found".to_string(),
        };
        write!(f, "{}", err_msg)
    }
}

impl From<subxt::Error> for RelayerError {
    fn from(value: subxt::Error) -> Self {
        Self::SubxtError(value)
    }
}

impl From<parity_scale_codec::Error> for RelayerError {
    fn from(value: parity_scale_codec::Error) -> Self {
        Self::CodecError(value)
    }
}

impl From<tokio::sync::mpsc::error::SendError<TheaMessage>> for RelayerError {
    fn from(value: SendError<TheaMessage>) -> Self {
        Self::TokioChannelError(value)
    }
}

impl From<ethers::abi::Error> for RelayerError {
    fn from(value: ethers::abi::Error) -> Self {
        Self::EthersAbiError(value)
    }
}

impl From<ethers::contract::ContractError<Provider<Ws>>> for RelayerError {
    fn from(value: ethers::contract::ContractError<Provider<Ws>>) -> Self {
        Self::EthersContractError(value)
    }
}

impl From<SignerMiddlewareError<Provider<Ws>, Wallet<SigningKey>>> for RelayerError {
    fn from(value: SignerMiddlewareError<Provider<Ws>, Wallet<SigningKey>>) -> Self {
        Self::EthersSignerMiddlewareError(value)
    }
}

impl From<ethers::providers::ProviderError> for RelayerError {
    fn from(value: ProviderError) -> Self {
        Self::EthersProviderError(value)
    }
}

impl From<WalletError> for RelayerError {
    fn from(value: WalletError) -> Self {
        Self::EthersWalletError(value)
    }
}
impl From<std::io::Error> for RelayerError {
     fn from(value: std::io::Error) -> Self {
         Self::IoError(value)
     }
}

impl From<subxt_signer::ecdsa::Error> for RelayerError {
    fn from(value: subxt_signer::ecdsa::Error) -> Self {
        Self::SubxtSignerError(value)
    }
}

