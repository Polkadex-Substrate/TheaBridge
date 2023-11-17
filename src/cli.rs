use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(parse(from_os_str))]
    pub thea_contract: PathBuf,
    #[structopt(
        short = "e",
        long = "eth-url",
        default_value = "https://ropsten.infura.io/v3/3d30527c65a144f082effabeaa0c778d"
    )]
    pub eth_url: String,
    #[structopt(
    short = "e",
    long = "sub-url",
    default_value = "localhost:9944"
    )]
    pub sub_url: String,
    #[structopt(
        short = "t",
        long = "thea-contract-address",
        default_value = "0x69F593B0F96EE94041422bF60208Ec6d007D909F"
    )]
    pub thea_contract_address: String,
    #[structopt(
        short = "s",
        long = "seed",
        default_value = "380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567da"
    )]
    pub seed: String,
}
