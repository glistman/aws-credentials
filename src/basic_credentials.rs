use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    credential_provider::{AwsCredentialProvider, AwsCredentials},
    errors::AwsCredentialsError,
};
pub struct AwsBasicCretendtialsProvider {
    credentials: AwsCredentials,
}

impl AwsBasicCretendtialsProvider {
    pub fn new(
        aws_access_key_id: &str,
        aws_secret_access_key: &str,
    ) -> Arc<RwLock<AwsBasicCretendtialsProvider>> {
        Arc::new(RwLock::new(AwsBasicCretendtialsProvider {
            credentials: AwsCredentials {
                aws_access_key_id: aws_access_key_id.to_string(),
                aws_secret_access_key: aws_secret_access_key.to_string(),
                aws_session_token: None,
            },
        }))
    }

    pub fn new_with_session_token(
        aws_access_key_id: &str,
        aws_secret_access_key: &str,
        aws_session_token: &str,
    ) -> Arc<RwLock<AwsBasicCretendtialsProvider>> {
        Arc::new(RwLock::new(AwsBasicCretendtialsProvider {
            credentials: AwsCredentials {
                aws_access_key_id: aws_access_key_id.to_string(),
                aws_secret_access_key: aws_secret_access_key.to_string(),
                aws_session_token: Some(aws_session_token.to_string()),
            },
        }))
    }
}

#[async_trait]
impl AwsCredentialProvider for AwsBasicCretendtialsProvider {
    async fn get_credentials(&self) -> Result<&AwsCredentials, AwsCredentialsError> {
        Ok(&self.credentials)
    }
}
