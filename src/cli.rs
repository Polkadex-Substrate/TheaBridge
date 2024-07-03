use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(short = "z", parse(from_os_str))]
    pub thea_contract: PathBuf,
    #[structopt(
        short = "e",
        long = "eth-url",
        default_value = "wss://sepolia.infura.io/ws/v3/93554318ae184575adc64c64e2aa7e0c"
    )]
    pub eth_url: String,
    #[structopt(short = "p", long = "sub-url", default_value = "wss://polkadex-mainnet-rpc.dwellir.com:443")]
    pub sub_url: String,
    #[structopt(
        short = "t",
        long = "thea-contract-address",
        default_value = "0xba39d2ead72ce331481f482cda2ef24fbda718d8"
    )]
    pub thea_contract_address: String,
    #[structopt(
    short = "n",
    long = "substrate-network-id"
    )]
    pub substrate_network_id: u8,
    #[structopt(
    short = "k",
    long = "evm-network-id"
    )]
    pub evn_network_id: u8,
    #[structopt(
        short = "s",
        long = "evm-seed",
        default_value = "c05c6ae125754dd17f36bcc5318498ce5c6c2f0e9e1116c68b77889a8be2ff02"
    )]
    pub evm_seed: String,
    #[structopt(
    short = "q",
    long = "sub-phase",
    default_value = "c05c6ae125754dd17f36bcc5318498ce5c6c2f0e9e1116c68b77889a8be2ff02"
    )]
    pub sub_phase: String,
}
