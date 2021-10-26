use reqwest::Error;

#[derive(Debug)]
pub enum AwsCredentialsError {
    AwsCredentialsEnvNotFound,
    RequestError(Error),
    CredentialsNotFound,
}
