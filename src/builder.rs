use crate::evmclient::EvmClient;
use crate::substrateclient::SubstrateClient;
use ethers::abi::{ethabi, Contract};
use std::path::PathBuf;

pub struct NoDestinationChain;
pub struct DestinationChain(String);
pub struct NoContract;
pub struct EVMContract(Contract);
pub struct NoSeed;
pub struct Seed(String);
pub struct NoTheaContractAddress;
pub struct TheaContractAddress(String);

pub struct Builder<Url, ContractFile, SeedString, ContractAddress> {
    chain_url: Url,
    contract: ContractFile,
    seed: SeedString,
    contract_address: ContractAddress,
}

impl Default for Builder<NoDestinationChain, NoContract, NoSeed, NoTheaContractAddress> {
    fn default() -> Builder<NoDestinationChain, NoContract, NoSeed, NoTheaContractAddress> {
        Builder {
            chain_url: NoDestinationChain,
            contract: NoContract,
            seed: NoSeed,
            contract_address: NoTheaContractAddress,
        }
    }
}

impl<Url, ContractFile, SeedString, ContractAddress>
    Builder<Url, ContractFile, SeedString, ContractAddress>
{
    pub fn chain_url(
        self,
        chain_url: String,
    ) -> Builder<DestinationChain, ContractFile, SeedString, ContractAddress> {
        Builder {
            chain_url: DestinationChain(chain_url),
            contract: self.contract,
            seed: self.seed,
            contract_address: self.contract_address,
        }
    }

    pub fn contract(
        self,
        contract_location: PathBuf,
    ) -> Builder<Url, EVMContract, SeedString, ContractAddress> {
        let log_abi_file = std::fs::File::open(contract_location).unwrap();
        let log_contract = ethabi::Contract::load(log_abi_file).unwrap();
        Builder {
            chain_url: self.chain_url,
            contract: EVMContract(log_contract),
            seed: self.seed,
            contract_address: self.contract_address,
        }
    }

    pub fn seed(self, seed: String) -> Builder<Url, ContractFile, Seed, ContractAddress> {
        Builder {
            chain_url: self.chain_url,
            contract: self.contract,
            seed: Seed(seed),
            contract_address: self.contract_address,
        }
    }

    pub fn contract_address(
        self,
        contract_address: String,
    ) -> Builder<Url, ContractFile, SeedString, TheaContractAddress> {
        Builder {
            chain_url: self.chain_url,
            contract: self.contract,
            seed: self.seed,
            contract_address: TheaContractAddress(contract_address),
        }
    }
}

impl Builder<DestinationChain, EVMContract, Seed, TheaContractAddress> {
    pub async fn build(self) -> EvmClient {
        EvmClient::new(
            self.chain_url.0,
            self.contract.0,
            self.seed.0,
            self.contract_address.0,
        )
        .await
    }
}

impl Builder<DestinationChain, NoContract, NoSeed, NoTheaContractAddress> {
    //FIXME: Take seed while building
    pub async fn build(self) -> SubstrateClient {
        SubstrateClient::initialize(self.chain_url.0).await.unwrap()
    }
}
