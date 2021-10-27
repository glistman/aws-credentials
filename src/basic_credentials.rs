use async_trait::async_trait;

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
    ) -> AwsBasicCretendtialsProvider {
        AwsBasicCretendtialsProvider {
            credentials: AwsCredentials {
                aws_access_key_id: aws_access_key_id.to_string(),
                aws_secret_access_key: aws_secret_access_key.to_string(),
                aws_session_token: None,
            },
        }
    }
}

#[async_trait]
impl AwsCredentialProvider for AwsBasicCretendtialsProvider {
    async fn get_credentials(&self) -> Result<&AwsCredentials, AwsCredentialsError> {
        Ok(&self.credentials)
    }
}
