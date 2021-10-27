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
pub struct AwsCointainerCredentials {
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

impl AwsCointainerCredentials {
    pub fn get_credentials(&self) -> AwsCredentials {
        AwsCredentials {
            aws_access_key_id: self.access_key_id.clone(),
            aws_secret_access_key: self.secret_access_key.clone(),
            aws_session_token: Some(self.token.clone()),
        }
    }
}

pub struct AwsContrainerCretendtialsProvider {
    pub credentials: Option<AwsCredentials>,
    pub credentials_url: Option<String>,
    pub ttl_in_seconds: u64,
    pub error: bool,
}

impl AwsCointainerCredentials {
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

impl AwsContrainerCretendtialsProvider {
    pub async fn new() -> Arc<RwLock<AwsContrainerCretendtialsProvider>> {
        let mut credentials: Option<AwsCredentials> = None;
        let url = AwsContrainerCretendtialsProvider::get_credentials_url();

        let mut ttl: u64 = 1;

        if let Some(credentials_url) = &url {
            if let Ok(aws_credentials) =
                AwsContrainerCretendtialsProvider::load_credentials(credentials_url).await
            {
                ttl = aws_credentials.get_ttl_in_seconds();
                credentials = Some(aws_credentials.get_credentials());
            }
        }

        let provider = Arc::new(RwLock::new(AwsContrainerCretendtialsProvider {
            credentials,
            credentials_url: url,
            ttl_in_seconds: ttl,
            error: false,
        }));

        let refresh_provider = provider.clone();

        tokio::spawn(async move {
            AwsContrainerCretendtialsProvider::execute_refresh_procedure(refresh_provider).await;
        });

        provider
    }

    pub async fn await_for_reload(&self) {
        if self.error {
            sleep(Duration::from_secs(1)).await;
        } else {
            sleep(Duration::from_secs(self.ttl_in_seconds)).await;
        }
    }

    fn get_credentials_url() -> Option<String> {
        std::env::var_os("AWS_CONTAINER_CREDENTIALS_RELATIVE_URI")
            .map(|path| format!("http://169.254.170.2{}", path.to_string_lossy()))
    }

    pub async fn load_credentials(
        url: &str,
    ) -> Result<AwsCointainerCredentials, AwsCredentialsError> {
        let aws_container_credentials = reqwest::get(url)
            .await
            .map_err(|error| AwsCredentialsError::RequestError(error))?
            .json::<AwsCointainerCredentials>()
            .await
            .map_err(|error| AwsCredentialsError::RequestError(error))?;

        Ok(aws_container_credentials)
    }

    pub async fn reload(&mut self) {
        if self.credentials_url.is_none() {
            self.credentials_url = AwsContrainerCretendtialsProvider::get_credentials_url();
        }

        if let Some(credentials_url) = &self.credentials_url {
            if let Ok(new_aws_container_credentials) =
                AwsContrainerCretendtialsProvider::load_credentials(&credentials_url).await
            {
                self.credentials = Some(new_aws_container_credentials.get_credentials());
                self.error = false;
            } else {
                self.error = true;
            }
        } else {
            self.error = true;
        }
    }

    pub async fn get_raw_credentials<'a>(
        &'a self,
    ) -> Result<&'a AwsCredentials, AwsCredentialsError> {
        match &self.credentials {
            Some(credentials) => Ok(credentials),
            None => Err(AwsCredentialsError::CredentialsNotFound),
        }
    }

    pub async fn execute_refresh_procedure(
        provider: Arc<RwLock<AwsContrainerCretendtialsProvider>>,
    ) {
        loop {
            let aws_credential_provider = provider.read().await;
            aws_credential_provider.await_for_reload().await;
            drop(aws_credential_provider);
            let mut aws_credential_provider = provider.write().await;
            aws_credential_provider.reload().await;
        }
    }
}

#[async_trait]
impl AwsCredentialProvider for AwsContrainerCretendtialsProvider {
    async fn get_credentials(&self) -> Result<&AwsCredentials, AwsCredentialsError> {
        self.get_raw_credentials().await
    }
}

#[cfg(test)]
mod tests {
    use crate::container_credentials::AwsCointainerCredentials;

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
        let credentials = serde_json::from_str::<AwsCointainerCredentials>(JSON_CREDENTIALS);
        assert_eq!(credentials.is_ok(), true);
    }
}
