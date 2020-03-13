use crate::{Parse, Reqwest, Result};
use reqwest::Client as HttpClient;
use snafu::ResultExt;
use url::Url;

#[derive(Debug)]
pub struct SimulationClient {
    /// IP address and port of simulation_server for sending API requests.
    base_url: Url,
    http_client: HttpClient,
}

impl SimulationClient {
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            http_client: HttpClient::new(),
        }
    }

    pub async fn create_execution_environment(
        &self,
        a: simulation_args::CreateExecutionEnvironment,
    ) -> Result<u64> {
        let url = self
            .base_url
            .join("/create-execution-environment")
            .context(Parse)?;

        let res = self
            .http_client
            .post(url)
            .json(&a)
            .send()
            .await
            .context(Reqwest)?
            .json::<u64>()
            .await
            .context(Reqwest)?;

        Ok(res)
    }
    pub async fn create_shard_block(&self, a: simulation_args::CreateShardBlock) -> Result<u64> {
        let url = self.base_url.join("/create-shard-block").context(Parse)?;

        let res = self
            .http_client
            .post(url)
            .json(&a)
            .send()
            .await
            .context(Reqwest)?
            .json::<u64>()
            .await
            .context(Reqwest)?;

        Ok(res)
    }
    pub async fn get_execution_environment(
        &self,
        a: simulation_args::GetExecutionEnvironment,
    ) -> Result<simulation_args::ExecutionEnvironment> {
        let url = self
            .base_url
            .join("/get-execution-environment")
            .context(Parse)?;

        let res = self
            .http_client
            .post(url)
            .json(&a)
            .send()
            .await
            .context(Reqwest)?
            .json::<simulation_args::ExecutionEnvironment>()
            .await
            .context(Reqwest)?;

        Ok(res)
    }
    pub async fn get_execution_environment_state(
        &self,
        a: simulation_args::GetExecutionEnvironmentState,
    ) -> Result<[u8; 32]> {
        let url = self
            .base_url
            .join("/get-execution-environment-state")
            .context(Parse)?;

        let res = self
            .http_client
            .post(url)
            .json(&a)
            .send()
            .await
            .context(Reqwest)?
            .json::<simulation_args::CustomSerializedReturnTypes>()
            .await
            .context(Reqwest)?;

        let simulation_args::CustomSerializedReturnTypes::Base64EncodedRoot(root) = res;
        Ok(root)
    }
    pub async fn get_shard_block(
        &self,
        a: simulation_args::GetShardBlock,
    ) -> Result<simulation_args::ShardBlock> {
        let url = self.base_url.join("/get-shard-block").context(Parse)?;

        let res = self
            .http_client
            .post(url)
            .json(&a)
            .send()
            .await
            .context(Reqwest)?
            .json::<simulation_args::ShardBlock>()
            .await
            .context(Reqwest)?;

        Ok(res)
    }
    pub async fn get_shard_state(
        &self,
        a: simulation_args::GetShardState,
    ) -> Result<simulation_args::ShardState> {
        let url = self.base_url.join("/get-shard-state").context(Parse)?;

        let res = self
            .http_client
            .post(url)
            .json(&a)
            .send()
            .await
            .context(Reqwest)?
            .json::<simulation_args::ShardState>()
            .await
            .context(Reqwest)?;

        Ok(res)
    }
}
