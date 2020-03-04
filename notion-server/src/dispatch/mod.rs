use eth2_types::eth_spec::EthSpec;
pub use simulation::{Error as SimulationError, Simulation};
pub use simulation_args;
use snafu::{OptionExt, ResultExt, Snafu};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use types as eth2_types;

/// Shorthand for result types returned from Dispatch.
pub type Result<V, E = Error> = std::result::Result<V, E>;

#[derive(Debug, Snafu)]
pub enum Error {
    /// Error calling "send" on Operation enum value
    // TODO: Handle these errors a bit more elegantly than is happening currently
    // (eg. wrap the underlying `std::sync::mpsc::SendError` -- this is a bit of a pain given
    //  that it uses generics)
    Send,

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
    CreateShardBlock(simulation_args::CreateShardBlock, Sender<Result<u64>>),
    GetExecutionEnvironment(
        simulation_args::GetExecutionEnvironment,
        Sender<Result<simulation_args::ExecutionEnvironment>>,
    ),
    GetExecutionEnvironmentState(simulation_args::GetExecutionEnvironmentState, Sender<Result<[u8; 32]>>),
    GetShardBlock(simulation_args::GetShardBlock, Sender<Result<simulation_args::ShardBlock>>),
    GetShardState(simulation_args::GetShardState, Sender<Result<simulation_args::ShardState>>),
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
                    reply.send(res).await.map_err(|_| Error::Send)?;
                }
                Operation::CreateShardBlock(args, mut reply) => {
                    let res = self.simulation.create_shard_block(args).context(Sim);
                    reply.send(res).await.map_err(|_| Error::Send)?;
                }
                Operation::GetExecutionEnvironment(args, mut reply) => {
                    let res = self.simulation.get_execution_environment(args).context(Sim);
                    reply.send(res).await.map_err(|_| Error::Send)?;
                }
                Operation::GetExecutionEnvironmentState(args, mut reply) => {
                    let res = self.simulation.get_execution_environment_state(args).context(Sim);
                    reply.send(res).await.map_err(|_| Error::Send)?;
                }
                Operation::GetShardBlock(args, mut reply) => {
                    let res = self.simulation.get_shard_block(args).context(Sim);
                    reply.send(res).await.map_err(|_| Error::Send)?;
                }
                Operation::GetShardState(args, mut reply) => {
                    let res = self.simulation.get_shard_state(args).context(Sim);
                    reply.send(res).await.map_err(|_| Error::Send)?;
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
            .await.map_err(|_| Error::Send)?;

        receiver.recv().await.context(Terminated)?
    }

    pub async fn create_shard_block(&mut self, arg: simulation_args::CreateShardBlock) -> Result<u64> {
        let (sender, mut receiver) = channel(1);

        self.sender.send(Operation::CreateShardBlock(arg, sender)).await.map_err(|_| Error::Send)?;

        receiver.recv().await.context(Terminated)?
    }

    pub async fn get_execution_environment(&mut self, arg: simulation_args::GetExecutionEnvironment) -> Result<simulation_args::ExecutionEnvironment> {
        let (sender, mut receiver) = channel(1);

        self.sender.send(Operation::GetExecutionEnvironment(arg, sender)).await.map_err(|_| Error::Send)?;

        receiver.recv().await.context(Terminated)?
    }

    pub async fn get_execution_environment_state(&mut self, arg: simulation_args::GetExecutionEnvironmentState) -> Result<[u8; 32]> {
        let (sender, mut receiver) = channel(1);

        self.sender.send(Operation::GetExecutionEnvironmentState(arg, sender)).await.map_err(|_| Error::Send)?;

        receiver.recv().await.context(Terminated)?
    }

    pub async fn get_shard_block(&mut self, arg: simulation_args::GetShardBlock) -> Result<simulation_args::ShardBlock> {
        let (sender, mut receiver) = channel(1);

        self.sender.send(Operation::GetShardBlock(arg, sender)).await.map_err(|_| Error::Send)?;

        receiver.recv().await.context(Terminated)?
    }

    pub async fn get_shard_state(&mut self, arg: simulation_args::GetShardState) -> Result<simulation_args::ShardState> {
        let (sender, mut receiver) = channel(1);

        self.sender.send(Operation::GetShardState(arg, sender)).await.map_err(|_| Error::Send)?;

        receiver.recv().await.context(Terminated)?
    }
}
