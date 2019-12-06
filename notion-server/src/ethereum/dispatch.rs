use tokio::sync::mpsc::{channel, Receiver, Sender};

use snafu::{OptionExt, ResultExt};

use super::{args, Error, Result, Simulation, Terminated};

#[derive(Debug)]
enum Operation {
    CreateExecutionEnvironment(args::CreateExecutionEnvironment, Sender<Result<u32>>),
    CreateShardBlock(args::CreateShardBlock, Sender<Result<u32>>),
    CreateShardChain(args::CreateShardChain, Sender<u32>),
    GetExecutionEnvironment(
        args::GetExecutionEnvironment,
        Sender<Result<args::ExecutionEnvironment>>,
    ),
    GetShardBlock(args::GetShardBlock, Sender<Result<args::ShardBlock>>),
    GetSimulationState(Sender<args::SimulationState>),
}

#[derive(Debug)]
pub struct Dispatch {
    simulation: Simulation,
    receiver: Receiver<Operation>,
}

impl Dispatch {
    pub fn new(simulation: Simulation) -> (Self, Handle) {
        let (sender, receiver) = channel(1);
        let handle = Handle(sender);

        let me = Dispatch {
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
                    let res = self.simulation.create_execution_environment(args);
                    reply.send(res).await;
                }
                Operation::CreateShardBlock(args, mut reply) => {
                    let res = self.simulation.create_shard_block(args);
                    reply.send(res).await;
                }
                Operation::CreateShardChain(args, mut reply) => {
                    let res = self.simulation.create_shard_chain(args);
                    reply.send(res).await;
                }
                Operation::GetExecutionEnvironment(args, mut reply) => {
                    let res = self.simulation.get_execution_environment(args);
                    reply.send(res).await;
                }
                Operation::GetShardBlock(args, mut reply) => {
                    let res = self.simulation.get_shard_block(args);
                    reply.send(res).await;
                }
                Operation::GetSimulationState(mut reply) => {
                    let res = self.simulation.simulation_state();
                    reply.send(res).await;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Handle(Sender<Operation>);

impl Handle {
    pub async fn create_execution_environment(
        &mut self,
        arg: args::CreateExecutionEnvironment,
    ) -> Result<u32> {
        let (sender, mut receiver) = channel(1);

        self.0
            .send(Operation::CreateExecutionEnvironment(arg, sender))
            .await;

        receiver.recv().await.context(Terminated)?
    }

    pub async fn create_shard_block(&mut self, arg: args::CreateShardBlock) -> Result<u32> {
        let (sender, mut receiver) = channel(1);

        self.0.send(Operation::CreateShardBlock(arg, sender)).await;

        receiver.recv().await.context(Terminated)?
    }

    pub async fn create_shard_chain(&mut self, arg: args::CreateShardChain) -> Result<u32> {
        let (sender, mut receiver) = channel(1);

        self.0.send(Operation::CreateShardChain(arg, sender)).await;

        receiver.recv().await.context(Terminated)
    }

    pub async fn execution_environment(
        &mut self,
        arg: args::GetExecutionEnvironment,
    ) -> Result<args::ExecutionEnvironment> {
        let (sender, mut receiver) = channel(1);

        self.0
            .send(Operation::GetExecutionEnvironment(arg, sender))
            .await;

        receiver.recv().await.context(Terminated)?
    }

    pub async fn shard_block(&mut self, arg: args::GetShardBlock) -> Result<args::ShardBlock> {
        let (sender, mut receiver) = channel(1);

        self.0.send(Operation::GetShardBlock(arg, sender)).await;

        receiver.recv().await.context(Terminated)?
    }

    pub async fn simulation_state(&mut self) -> Result<args::SimulationState> {
        let (sender, mut receiver) = channel(1);

        self.0.send(Operation::GetSimulationState(sender)).await;

        receiver.recv().await.context(Terminated)
    }
}
