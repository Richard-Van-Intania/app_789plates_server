use crate::constants::BUCKET_NAME;
use aws_sdk_s3::{presigning::PresigningConfig, Client, Error};
use std::time::Duration;

pub async fn remove_object(client: &Client, key: String) -> Result<(), Error> {
    client
        .delete_object()
        .bucket(BUCKET_NAME)
        .key(key)
        .send()
        .await?;
    Ok(())
}

pub async fn generate_presigned_url(
    client: &Client,
    key: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let expires_in = Duration::from_secs(7200);
    let presigned_request = client
        .put_object()
        .bucket(BUCKET_NAME)
        .key(key)
        .presigned(PresigningConfig::expires_in(expires_in)?)
        .await?;
    Ok(presigned_request.uri().to_string())
}
