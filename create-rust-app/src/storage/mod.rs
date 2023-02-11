use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use aws_sdk_s3::model::{Delete, ObjectIdentifier};
use aws_sdk_s3::presigning::config::PresigningConfig;
use aws_sdk_s3::types::ByteStream;
//use aws_sdk_s3::types::SdkError::*;
use aws_sdk_s3::{Client, Config, Endpoint};
use aws_types::region::Region;
use aws_types::Credentials;
//use base64;
use http::{HeaderMap, Uri};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub use attachment::{Attachment, AttachmentData};
pub use attachment_blob::AttachmentBlob;

mod attachment;
mod attachment_blob;
mod schema;

#[tsync::tsync]
type ID = i32;

#[tsync::tsync]
#[cfg(not(feature = "database_sqlite"))]
type Utc = chrono::DateTime<chrono::Utc>;
#[cfg(feature = "database_sqlite")]
type Utc = chrono::NaiveDateTime;

#[derive(Clone)]
pub struct Storage {
    client: Option<Client>,
    bucket: String,
    host: String,
}

pub struct UploadURI {
    pub headers: HeaderMap,
    pub uri: Uri,
}
impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}
impl Storage {
    pub async fn download(&self, key: String, to_path: PathBuf) -> Result<(), String> {
        let client = self.client_or_error()?;

        let response = client
            .get_object()
            .bucket(&self.bucket)
            .key(key.clone())
            .send()
            .await
            .map_err(|err| {
                self.error_string("Could not download object", key.clone(), err.to_string())
            })?;

        let data = response.body.collect().await.map_err(|err| {
            self.error_string("Could not download object", key.clone(), err.to_string())
        })?;

        let mut file = File::create(to_path).await.map_err(|err| {
            self.error_string("Could not download object", key.clone(), err.to_string())
        })?;

        file.write_all(&data.into_bytes()).await.map_err(|err| {
            self.error_string("Could not download object", key.clone(), err.to_string())
        })?;

        Ok(())
    }

    /// if `expires_in` is `None`, then we assume the bucket is publicly accessible and return the
    /// public URL. For this to work, you have to make sure the bucket's policy allows public access.
    pub async fn download_uri(
        &self,
        key: String,
        expires_in: Option<Duration>,
    ) -> Result<String, String> {
        if expires_in.is_none() {
            let host = self.host.clone();
            let host = if host.ends_with('/') {
                host
            } else {
                format!("{host}/")
            };
            let bucket = &self.bucket;
            return Ok(format!("{host}{bucket}/{key}"));
        }
        let expires_in = expires_in.unwrap();

        let client = self.client_or_error()?;

        let response = client
            .get_object()
            .bucket(&self.bucket)
            .key(key.clone())
            .presigned(PresigningConfig::expires_in(expires_in).map_err(|err| {
                self.error_string(
                    "Could not retrieve download URI",
                    key.clone(),
                    err.to_string(),
                )
            })?)
            .await
            .map_err(|err| {
                self.error_string(
                    "Could not retrieve download URI",
                    key.clone(),
                    err.to_string(),
                )
            })?;

        Ok(response.uri().to_string())
    }

    pub async fn upload(
        &self,
        key: String,
        bytes: Vec<u8>,
        content_type: String,
        _content_md5: String,
    ) -> Result<(), String> {
        let stream = ByteStream::from(bytes);

        let client = self.client_or_error()?;

        client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(stream)
            .content_type(content_type)
            // TODO: Error { code: \"InvalidDigest\", message: \"The Content-Md5 you specified is not valid.\", request_id: \"16DBB0A878146F1A\" }
            // .content_md5(base64::encode(content_md5))
            .send()
            .await
            .map_err(|err| {
                self.error_string("Could not upload object", key.clone(), err.to_string())
            })?;

        Ok(())
    }

    pub async fn upload_uri(&self, key: String, expires_in: Duration) -> Result<UploadURI, String> {
        let client = self.client_or_error()?;

        let response = client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .presigned(PresigningConfig::expires_in(expires_in).map_err(|err| {
                self.error_string(
                    "Could not retrieve upload URI",
                    key.clone(),
                    err.to_string(),
                )
            })?)
            .await
            .map_err(|err| {
                self.error_string("Could not retrieve upload URI", key, err.to_string())
            })?;

        let upload_uri = UploadURI {
            uri: response.uri().clone(),
            headers: response.headers().clone(),
        };

        Ok(upload_uri)
    }

    pub async fn delete(&self, key: String) -> Result<(), String> {
        let client = self.client_or_error()?;

        client
            .delete_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|err| {
                self.error_string("Could not delete object", key.clone(), err.to_string())
            })?;

        Ok(())
    }

    pub async fn delete_many(&self, keys: Vec<String>) -> Result<(), String> {
        let client = self.client_or_error()?;

        let ids = keys
            .iter()
            .map(|k| {
                ObjectIdentifier::builder()
                    .set_key(Some(k.to_string()))
                    .build()
            })
            .collect::<Vec<ObjectIdentifier>>();
        let delete = Delete::builder().set_objects(Some(ids)).build();

        client
            .delete_objects()
            .bucket(&self.bucket)
            .delete(delete)
            .send()
            .await
            .map_err(|err| {
                self.error_string(
                    "Could not delete objects",
                    format!("{keys:#?}"),
                    err.to_string(),
                )
            })?;

        Ok(())
    }

    fn error_string(&self, message: &'static str, key: String, error: String) -> String {
        let bucket = &self.bucket;
        format!("{message} (bucket: '{bucket}', key: '{key}', error: '{error}')")
    }

    fn client_or_error(&self) -> Result<&Client, String> {
        self.client.as_ref().ok_or_else(|| {
            "The storage is not available; did you set the right environment variables?".to_string()
        })
    }

    fn check_environment_variables() {
        let vars = vec![
            "S3_HOST",
            "S3_REGION",
            "S3_BUCKET",
            "S3_ACCESS_KEY_ID",
            "S3_SECRET_ACCESS_KEY",
        ];

        let unset_vars = vars
            .into_iter()
            .filter(|v| std::env::var(v).is_err())
            .collect::<Vec<_>>();

        if !unset_vars.is_empty() {
            println!(
                "Warning: Storage disabled; the following variables must be set: {}",
                unset_vars.join(", ")
            );
        }
    }

    fn init(
        host: String,
        region: String,
        access_key_id: String,
        secret_access_key: String,
    ) -> Result<Option<Client>, String> {
        Storage::check_environment_variables();

        let host = host;
        let region = Region::new(region);
        let s3_config = Config::builder()
            .region(region)
            .endpoint_resolver(Endpoint::immutable(Uri::from_str(host.as_str()).map_err(
                |err| {
                    let error = err.to_string();
                    format!("Could not initialize storage (error: '{error}')")
                },
            )?))
            .credentials_provider(Credentials::new(
                access_key_id,
                secret_access_key,
                None,
                None,
                "UNNAMED_PROVIDER",
            ))
            .build();
        let client = Client::from_conf(s3_config);

        Ok(Some(client))
    }

    pub fn new() -> Storage {
        let host = std::env::var("S3_HOST").unwrap_or_else(|_| "".to_string());
        let region = std::env::var("S3_REGION").unwrap_or_else(|_| "".to_string());
        let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "".to_string());
        let access_key_id = std::env::var("S3_ACCESS_KEY_ID").unwrap_or_else(|_| "".to_string());
        let secret_access_key =
            std::env::var("S3_SECRET_ACCESS_KEY").unwrap_or_else(|_| "".to_string());

        let client =
            Storage::init(host.clone(), region, access_key_id, secret_access_key).unwrap_or(None);

        Storage {
            client,
            bucket,
            host,
        }
    }
}
