# AWS Container Credentials
extract aws credentials from container

## Usage

```bash
use std::sync::Arc;

use aws_credentials::{
    container_credentials::AwsContrainerCretendtialsProvider,
    credential_provider::AwsCredentialProvider,
};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let aws_contrainer_cretendtials_provider =
        Arc::new(RwLock::new(AwsContrainerCretendtialsProvider::new().await));

    let aws_contrainer_cretendtials_provider_refresh = aws_contrainer_cretendtials_provider.clone();

    tokio::spawn(async move {
        AwsContrainerCretendtialsProvider::execute_refresh_procedure(
            aws_contrainer_cretendtials_provider_refresh,
        )
        .await;
    });

    let aws_contrainer_cretendtials_provider = aws_contrainer_cretendtials_provider.read().await;
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