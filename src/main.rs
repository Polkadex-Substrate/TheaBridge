use crate::builder::Builder;
use crate::cli::Cli;
use crate::relayer::RelayerBuilder;
use structopt::StructOpt;
use crate::error::RelayerError;

mod builder;
mod cli;
mod evmclient;
mod relayer;
mod substrateclient;
mod traits;
#[cfg(test)]
mod test;
pub mod error;

#[tokio::main]
async fn main() -> Result<(), RelayerError> {
    env_logger::init();
    let opt: Cli = cli::Cli::from_args();
    let evm_client = Builder::default()
        .chain_url(opt.eth_url)
        .contract_address(opt.thea_contract_address)
        .seed(opt.evm_seed)
        .contract(opt.thea_contract)?
        .build()
        .await?;
    let substrate_client = Builder::default().chain_url(opt.sub_url).build().await?;
    let mut relayer = RelayerBuilder::default()
        .evm_client(evm_client)
        .substrate_client(substrate_client)
        .build();
    relayer.run().await?;
    Ok(())
}
