use crate::builder::Builder;
use crate::cli::Cli;
use structopt::StructOpt;
use crate::relayer::RelayerBuilder;

mod builder;
mod cli;
mod evmclient;
mod relayer;
mod substrateclient;
mod traits;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt: Cli = cli::Cli::from_args();
    let evm_client = Builder::default()
        .chain_url(opt.eth_url)
        .contract_address(opt.thea_contract_address)
        .seed(opt.seed)
        .contract(opt.thea_contract)
        .build()
        .await;
    let substrate_client = Builder::default()
        .chain_url(opt.sub_url)
        .build()
        .await;
    let mut relayer = RelayerBuilder::default()
        .evm_client(evm_client)
        .substrate_client(substrate_client)
        .build();
    relayer.run().await?;
    Ok(())
}
