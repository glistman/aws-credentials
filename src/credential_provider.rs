use crate::errors::AwsCredentialsError;
use async_trait::async_trait;

#[async_trait]
pub trait AwsCredentialProvider {
    async fn get_credentials(&self) -> Result<&AwsCredentials, AwsCredentialsError>;
}

#[derive(Debug)]
pub struct AwsCredentials {
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
    pub aws_session_token: Option<String>,
}
