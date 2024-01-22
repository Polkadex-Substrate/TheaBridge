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
    #[structopt(short = "p", long = "sub-url", default_value = "ws://localhost:9944")]
    pub sub_url: String,
    #[structopt(
        short = "t",
        long = "thea-contract-address",
        default_value = "0xc5C4D1D4B2bEAd517863f4969cbb38A42aD1b11C"
    )]
    pub thea_contract_address: String,
    #[structopt(
        short = "s",
        long = "seed",
        default_value = "c05c6ae125754dd17f36bcc5318498ce5c6c2f0e9e1116c68b77889a8be2ff02"
    )]
    pub seed: String
}
