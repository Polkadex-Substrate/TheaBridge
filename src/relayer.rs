use crate::evmclient::EvmClient;
use crate::substrateclient::SubstrateClient;
use crate::traits::{Channel, TheaMessage};

pub struct NoEvmClient;
pub struct EvmClientA(EvmClient);
pub struct NoSubstrateClient;
pub struct SubstrateClientA(SubstrateClient);

pub struct RelayerBuilder<EvmClientX, SubstrateClientX> {
    evm_client: EvmClientX,
    substrate_client: SubstrateClientX,
}

impl Default for RelayerBuilder<NoEvmClient, NoSubstrateClient> {
    fn default() -> Self {
        RelayerBuilder {
            evm_client: NoEvmClient,
            substrate_client: NoSubstrateClient,
        }
    }
}

impl<EvmClientX, SubstrateClientX> RelayerBuilder<EvmClientX, SubstrateClientX> {
    pub fn evm_client(self, evm_client: EvmClient) -> RelayerBuilder<EvmClientA, SubstrateClientX> {
        RelayerBuilder {
            evm_client: EvmClientA(evm_client),
            substrate_client: self.substrate_client,
        }
    }

    pub fn substrate_client(
        self,
        substrate_client: SubstrateClient,
    ) -> RelayerBuilder<EvmClientX, SubstrateClientA> {
        RelayerBuilder {
            evm_client: self.evm_client,
            substrate_client: SubstrateClientA(substrate_client),
        }
    }
}

impl RelayerBuilder<EvmClientA, SubstrateClientA> {
    pub fn build(self) -> Relayer {
        Relayer {
            evm_client: self.evm_client.0,
            substrate_client: self.substrate_client.0,
        }
    }
}

pub struct Relayer {
    evm_client: EvmClient,
    substrate_client: SubstrateClient,
}

impl Relayer {
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut evm_deposit_channel = Channel::<TheaMessage>::new();
        // spawn following tasks
        let evm_client = self.evm_client.clone();
        let sender = evm_deposit_channel.sender().clone();
        tokio::spawn(async move {
            evm_client
                .subscribe_deposit_events_stream(sender)
                .await
                .unwrap();
        });
        let evm_client = self.evm_client.clone();
        let sender = evm_deposit_channel.sender().clone();
        tokio::spawn(async move {
            evm_client
                .subscribe_ob_deposit_events_stream(sender)
                .await
                .unwrap();
        });
        let substrate_client = self.substrate_client.clone();
        let sender = evm_deposit_channel.sender().clone();
        tokio::spawn(async move {
            substrate_client
                .subscribe_substrate_event_stream(sender)
                .await
                .unwrap();
        });
        loop {
            if let Some(message) = evm_deposit_channel.receiver.recv().await {
                match message {
                    TheaMessage::EvmDeposit(deposit) => {
                        self.substrate_client.handle_deposit(deposit).await?;
                    }
                    TheaMessage::ObEvmDeposit(deposit) => {
                        self.substrate_client.handle_ob_deposit(deposit).await?;
                    }
                    TheaMessage::SubstrateMessage(message) => {
                        self.evm_client.handle_substrate_message(message).await?;
                    }
                }
            }
        }
    }
}
