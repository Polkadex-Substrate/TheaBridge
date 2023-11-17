use crate::traits::VerificationMode;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(parse(from_os_str))]
    pub thea_contract: PathBuf,
    #[structopt(short = "e", long = "eth-url")]
    pub eth_url: String,
    #[structopt(short = "e", long = "sub-url", default_value = "localhost:9944")]
    pub sub_url: String,
    #[structopt(short = "t", long = "thea-contract-address")]
    pub thea_contract_address: String,
    #[structopt(short = "s", long = "seed")]
    pub seed: String,
    #[structopt(short = "vs", long = "vrf-seed")]
    pub vrf_seed: String,

    #[structopt(short = "m", long = "mode")]
    pub verification_mode: VerificationMode,
}
