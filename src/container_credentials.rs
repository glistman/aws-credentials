use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::{sync::RwLock, time::sleep};

use crate::{
    credential_provider::{AwsCredentialProvider, AwsCredentials},
    errors::AwsCredentialsError,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct AwsContainerCredentials {
    #[serde(alias = "RoleArn")]
    pub role_arn: String,
    #[serde(alias = "AccessKeyId")]
    pub access_key_id: String,
    #[serde(alias = "SecretAccessKey")]
    pub secret_access_key: String,
    #[serde(alias = "Token")]
    pub token: String,
    #[serde(alias = "Expiration")]
    pub expiration: DateTime<Utc>,
}

impl AwsContainerCredentials {
    pub fn get_credentials(&self) -> AwsCredentials {
        AwsCredentials {
            aws_access_key_id: self.access_key_id.clone(),
            aws_secret_access_key: self.secret_access_key.clone(),
            aws_session_token: Some(self.token.clone()),
        }
    }
}

pub struct AwsContainerCredentialsProvider {
    pub credentials: Option<AwsCredentials>,
    pub credentials_url: Option<String>,
    pub ttl_in_seconds: u64,
    pub error: bool,
}

impl AwsContainerCredentials {
    pub fn get_ttl_in_seconds(&self) -> u64 {
        let duration = self.expiration.signed_duration_since(Utc::now());
        let ttl = duration.num_seconds();
        if ttl < 0 {
            0
        } else {
            ttl as u64
        }
    }
}

impl AwsContainerCredentialsProvider {
    pub async fn new() -> Arc<RwLock<AwsContainerCredentialsProvider>> {
        let mut credentials: Option<AwsCredentials> = None;
        let url = AwsContainerCredentialsProvider::get_credentials_url();

        let mut ttl: u64 = 1;

        if let Some(credentials_url) = &url {
            if let Ok(aws_credentials) =
                AwsContainerCredentialsProvider::load_credentials(credentials_url).await
            {
                ttl = aws_credentials.get_ttl_in_seconds();
                credentials = Some(aws_credentials.get_credentials());
            }
        }

        let provider = Arc::new(RwLock::new(AwsContainerCredentialsProvider {
            credentials,
            credentials_url: url,
            ttl_in_seconds: ttl,
            error: false,
        }));

        let refresh_provider = provider.clone();

        tokio::spawn(async move {
            AwsContainerCredentialsProvider::execute_refresh_procedure(refresh_provider).await;
        });

        provider
    }

    pub async fn time_to_await(&self) -> Duration {
        if self.error {
            Duration::from_secs(1)
        } else {
            Duration::from_secs(self.ttl_in_seconds)
        }
    }

    fn get_credentials_url() -> Option<String> {
        std::env::var_os("AWS_CONTAINER_CREDENTIALS_RELATIVE_URI")
            .map(|path| format!("http://169.254.170.2{}", path.to_string_lossy()))
    }

    pub async fn load_credentials(
        url: &str,
    ) -> Result<AwsContainerCredentials, AwsCredentialsError> {
        let aws_container_credentials = reqwest::get(url)
            .await
            .map_err(AwsCredentialsError::RequestError)?
            .json::<AwsContainerCredentials>()
            .await
            .map_err(AwsCredentialsError::RequestError)?;

        Ok(aws_container_credentials)
    }

    pub async fn reload(&mut self) {
        if self.credentials_url.is_none() {
            self.credentials_url = AwsContainerCredentialsProvider::get_credentials_url();
        }
        let mut error = true;

        if let Some(credentials_url) = &self.credentials_url {
            if let Ok(new_aws_container_credentials) =
                AwsContainerCredentialsProvider::load_credentials(credentials_url).await
            {
                self.credentials = Some(new_aws_container_credentials.get_credentials());
                error = false;
            }
        }

        self.error = error;
    }

    pub async fn get_raw_credentials(&self) -> Result<&AwsCredentials, AwsCredentialsError> {
        match &self.credentials {
            Some(credentials) => Ok(credentials),
            None => Err(AwsCredentialsError::CredentialsNotFound),
        }
    }

    pub async fn execute_refresh_procedure(provider: Arc<RwLock<AwsContainerCredentialsProvider>>) {
        loop {
            let aws_credential_provider = provider.read().await;
            let time_to_await = aws_credential_provider.time_to_await().await;
            drop(aws_credential_provider);
            sleep(time_to_await).await;
            let mut aws_credential_provider = provider.write().await;
            aws_credential_provider.reload().await;
        }
    }
}

#[async_trait]
impl AwsCredentialProvider for AwsContainerCredentialsProvider {
    async fn get_credentials(&self) -> Result<&AwsCredentials, AwsCredentialsError> {
        self.get_raw_credentials().await
    }
}

#[cfg(test)]
mod tests {
    use crate::container_credentials::AwsContainerCredentials;

    const JSON_CREDENTIALS: &str = "
    {
        \"RoleArn\": \"arn:aws:iam::1234:role/task-role\",
        \"AccessKeyId\": \"ASIAYFVP67GZJEXAMPLE\",
        \"SecretAccessKey\": \"537N0my6Yv3UO48SRxfk6EXAMPLE\",
        \"Token\": \"IQoJb3JpZ2luX2VjEMTEXAMPLE\",
        \"Expiration\": \"2021-10-25T17:46:19Z\"
    }";

    #[test]
    fn json_response() {
        let credentials = serde_json::from_str::<AwsContainerCredentials>(JSON_CREDENTIALS);
        assert_eq!(credentials.is_ok(), true);
    }
}
