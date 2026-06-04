use tokio::time::{Duration, sleep};
use tracing::warn;

use crate::domain::{
    AgentId, FleetCommand, FleetUnit, JobId, JobSubmission, NewFleetUnit, format_job_id,
};

pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        let base_url = base_url.trim_end_matches('/').to_owned();
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }

    pub async fn register_with_retry(
        &self,
        agent_name: &str,
    ) -> Result<FleetUnit, Box<dyn std::error::Error>> {
        let url = format!("{}/fleet", self.base_url);
        let mut attempts = 0;

        loop {
            match self
                .client
                .post(&url)
                .json(&NewFleetUnit {
                    name: agent_name.to_owned(),
                })
                .send()
                .await
            {
                Ok(response) => {
                    let agent = response.error_for_status()?.json::<FleetUnit>().await?;
                    return Ok(agent);
                }
                Err(error) => {
                    attempts += 1;
                    warn!(%error, attempts, "failed to register, retrying");
                    if attempts >= 10 {
                        return Err(error.into());
                    }
                    sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }

    pub async fn next_command(
        &self,
        agent_id: AgentId,
    ) -> Result<Option<FleetCommand>, reqwest::Error> {
        let url = format!("{}/fleet/{}/commands/next", self.base_url, agent_id);
        let response = self.client.get(&url).send().await?.error_for_status()?;

        if response.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(None);
        }

        response.json::<FleetCommand>().await.map(Some)
    }

    pub async fn submit_result(
        &self,
        agent_id: AgentId,
        job_id: JobId,
        result: String,
    ) -> Result<(), reqwest::Error> {
        let url = format!(
            "{}/fleet/{}/jobs/{}/submit",
            self.base_url,
            agent_id,
            format_job_id(job_id)
        );

        self.client
            .post(&url)
            .json(&JobSubmission { result })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
