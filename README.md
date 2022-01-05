# AWS Container Credentials
extract aws credentials from container

## Usage

```bash
use aws_credentials::{
    container_credentials::AwsContrainerCretendtialsProvider,
    credential_provider::AwsCredentialProvider,
};

#[tokio::main]
async fn main() {
    let aws_container_credentials_provider = AwsContainerCredentialsProvider::new().await;
    let aws_container_credentials_provider = aws_contrainer_cretendtials_provider.read().await;
    let credentials_result = aws_contrainer_cretendtials_provider.get_credentials().await;

    match credentials_result {
        Ok(credentials) => println!("{:?}", credentials),
        Err(error) => println!("{:?}", error),
    }
}
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License
[MIT](https://choosealicense.com/licenses/mit/)