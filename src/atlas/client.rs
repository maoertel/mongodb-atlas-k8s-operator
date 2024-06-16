use diqwest::WithDigestAuth;
use reqwest::Client;
use reqwest::StatusCode;
use serde::Deserialize;

use crate::atlas::error::Error;
use crate::atlas::error::Result;
use crate::crd::AtlasUser;

const ATLAS_API_URL: &str = "https://cloud.mongodb.com/api/atlas/v2/users";
pub(crate) const ATLAS_API_CONTENT_TYPE: &str = "application/vnd.atlas.2023-02-01+json";

pub struct AtlasClient {
    client: Client,
    api_key: String,
    api_secret: String,
}

impl AtlasClient {
    pub fn new(client: Client, api_key: String, api_secret: String) -> Result<AtlasClient> {
        Ok(Self {
            client,
            api_key,
            api_secret,
        })
    }

    pub(crate) async fn create_atlas_user(&self, atlas_user: &AtlasUser) -> Result<()> {
        let response = self
            .client
            .post(ATLAS_API_URL)
            .json(&atlas_user.spec)
            .send_with_digest_auth(&self.api_key, &self.api_secret)
            .await?;

        match response.status() {
            StatusCode::OK => handle_ok_response(response).await,
            status => handle_error(status, response).await,
        }
    }
}

async fn handle_ok_response<A>(response: reqwest::Response) -> Result<A>
where
    A: for<'de> Deserialize<'de>,
{
    let content = response.bytes().await?;
    let cluster_details = serde_json::from_slice(&content)?;
    Ok(cluster_details)
}

async fn handle_error<A>(status: StatusCode, response: reqwest::Response) -> Result<A> {
    let message = response.text().await?;
    Err(Error::Api { status, message })
}
