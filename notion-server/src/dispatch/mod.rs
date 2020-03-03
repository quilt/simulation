use eth2_types::eth_spec::EthSpec;
pub use simulation::{args, Error as SimulationError, Simulation};
pub use simulation_args;
use snafu::{OptionExt, ResultExt, Snafu};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use types as eth2_types;

/// Shorthand for result types returned from Dispatch.
pub type Result<V, E = Error> = std::result::Result<V, E>;

#[derive(Debug, Snafu)]
pub enum Error {
    /// Simulation error
    // Called "Sim" instead of "Simulation" to prevent name collision because Snafu auto-generates
    // context selectors with the same name as the enum values
    Sim {
        source: SimulationError,
    },
    /// Operation was cancelled because the simulation is shutting down.
    Terminated,
}

#[derive(Debug)]
enum Operation {
    CreateExecutionEnvironment(simulation_args::CreateExecutionEnvironment, Sender<Result<u64>>),
    CreateShardBlock(args::CreateShardBlock, Sender<Result<u64>>),
    GetExecutionEnvironment(
        args::GetExecutionEnvironment,
        Sender<Result<args::ExecutionEnvironment>>,
    ),
    GetExecutionEnvironmentState(args::GetExecutionEnvironmentState, Sender<Result<[u8; 32]>>),
    GetShardBlock(args::GetShardBlock, Sender<Result<args::ShardBlock>>),
    GetShardState(args::GetShardState, Sender<Result<args::ShardState>>),
}

#[derive(Debug)]
pub struct Dispatch<T>
where T: EthSpec,
{
    simulation: Simulation<T>,
    receiver: Receiver<Operation>,
}

impl<T: EthSpec> Dispatch<T> {
    pub fn new(simulation: Simulation<T>) -> (Self, Handle) {
        let (sender, receiver) = channel(1);
        let handle = Handle {
            sender
        };

        let me: Dispatch<T> = Dispatch {
            simulation,
            receiver,
        };

        (me, handle)
    }

    pub async fn run(mut self) -> Result<()> {
        eprintln!("Simulation Running: {:?}", std::thread::current().id());
        while let Some(op) = self.receiver.recv().await {
            match op {
                Operation::CreateExecutionEnvironment(args, mut reply) => {
                    let res = self.simulation.create_execution_environment(args).context(Sim);
                    reply.send(res).await;
                }
                Operation::CreateShardBlock(args, mut reply) => {
                    let res = self.simulation.create_shard_block(args).context(Sim);
                    reply.send(res).await;
                }
                Operation::GetExecutionEnvironment(args, mut reply) => {
                    let res = self.simulation.get_execution_environment(args).context(Sim);
                    reply.send(res).await;
                }
                Operation::GetExecutionEnvironmentState(args, mut reply) => {
                    let res = self.simulation.get_execution_environment_state(args).context(Sim);
                    reply.send(res).await;
                }
                Operation::GetShardBlock(args, mut reply) => {
                    let res = self.simulation.get_shard_block(args).context(Sim);
                    reply.send(res).await;
                }
                Operation::GetShardState(args, mut reply) => {
                    let res = self.simulation.get_shard_state(args).context(Sim);
                    reply.send(res).await;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Handle {
    sender: Sender<Operation>,
}

impl Handle {
    pub async fn create_execution_environment(
        &mut self,
        arg: simulation_args::CreateExecutionEnvironment,
    ) -> Result<u64> {
        let (sender, mut receiver) = channel(1);

        self.sender
            .send(Operation::CreateExecutionEnvironment(arg, sender))
            .await;

        receiver.recv().await.context(Terminated)?
    }

    pub async fn create_shard_block(&mut self, arg: args::CreateShardBlock) -> Result<u64> {
        let (sender, mut receiver) = channel(1);

        self.sender.send(Operation::CreateShardBlock(arg, sender)).await;

        receiver.recv().await.context(Terminated)?
    }

    pub async fn get_execution_environment(&mut self, arg: args::GetExecutionEnvironment) -> Result<args::ExecutionEnvironment> {
        let (sender, mut receiver) = channel(1);

        self.sender.send(Operation::GetExecutionEnvironment(arg, sender)).await;

        receiver.recv().await.context(Terminated)?
    }

    pub async fn get_execution_environment_state(&mut self, arg: args::GetExecutionEnvironmentState) -> Result<[u8; 32]> {
        let (sender, mut receiver) = channel(1);

        self.sender.send(Operation::GetExecutionEnvironmentState(arg, sender)).await;

        receiver.recv().await.context(Terminated)?
    }

    pub async fn get_shard_block(&mut self, arg: args::GetShardBlock) -> Result<args::ShardBlock> {
        let (sender, mut receiver) = channel(1);

        self.sender.send(Operation::GetShardBlock(arg, sender)).await;

        receiver.recv().await.context(Terminated)?
    }

    pub async fn get_shard_state(&mut self, arg: args::GetShardState) -> Result<args::ShardState> {
        let (sender, mut receiver) = channel(1);

        self.sender.send(Operation::GetShardState(arg, sender)).await;

        receiver.recv().await.context(Terminated)?
    }
}
