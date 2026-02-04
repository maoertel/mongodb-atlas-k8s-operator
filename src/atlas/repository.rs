use std::sync::Arc;

use reqwest::{header, Client, Response, StatusCode};
use serde::Deserialize;

use crate::atlas::error::{Error, Result};
use crate::atlas::user_request::UserRequest;
use crate::atlas::user_response::UserResponse;

const ATLAS_API_CONTENT_TYPE_2025_02_19: &str = "application/vnd.atlas.2025-02-19+json";
const ATLAS_API_V2_BASE_URL: &str = "https://cloud.mongodb.com/api/atlas/v2";

/// Repository for interacting with the MongoDB Atlas Admin API v2
pub struct AtlasUserRepository {
    client: Client,
    access_token: Arc<str>,
}

impl AtlasUserRepository {
    /// Creates a new AtlasUserRepository with a bearer token for authentication
    pub fn new(access_token: Arc<str>) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json; charset=utf-8"),
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static(ATLAS_API_CONTENT_TYPE_2025_02_19),
        );

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self { client, access_token })
    }

    /// Invites a new user to the Atlas organization
    pub async fn invite_atlas_user(&self, org_id: &str, user: &UserRequest<'_>) -> Result<UserResponse> {
        let url = format!("{}/orgs/{}/users", ATLAS_API_V2_BASE_URL, org_id);

        let response = self
            .client
            .post(&url)
            .bearer_auth(self.access_token.as_ref())
            .json(user)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK | StatusCode::CREATED => handle_ok_response(response).await,
            status => handle_error(status, response).await,
        }
    }

    /// Updates an existing user in the Atlas organization
    pub async fn update_atlas_user(&self, org_id: &str, user_id: &str, user: &UserRequest<'_>) -> Result<UserResponse> {
        let url = format!("{}/orgs/{}/users/{}", ATLAS_API_V2_BASE_URL, org_id, user_id);

        let response = self
            .client
            .patch(&url)
            .bearer_auth(self.access_token.as_ref())
            .json(user)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => handle_ok_response(response).await,
            status => handle_error(status, response).await,
        }
    }

    /// Deletes a user from the Atlas organization
    pub async fn delete_atlas_user_from_org(&self, org_id: &str, user_id: &str) -> Result<()> {
        let url = format!("{}/orgs/{}/users/{}", ATLAS_API_V2_BASE_URL, org_id, user_id);

        let response = self
            .client
            .delete(&url)
            .bearer_auth(self.access_token.as_ref())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => Ok(()),
            status => handle_error(status, response).await,
        }
    }

    /// Gets a user from the Atlas organization by user ID
    pub async fn get_atlas_user(&self, org_id: &str, user_id: &str) -> Result<UserResponse> {
        let url = format!("{}/orgs/{}/users/{}", ATLAS_API_V2_BASE_URL, org_id, user_id);

        let response = self
            .client
            .get(&url)
            .bearer_auth(self.access_token.as_ref())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => handle_ok_response(response).await,
            StatusCode::NOT_FOUND => Err(Error::AtlasUserNotFound {
                user_id: user_id.to_string(),
                org_id: org_id.to_string(),
            }),
            status => handle_error(status, response).await,
        }
    }

    /// Finds a user in the Atlas organization by username (email)
    pub async fn find_atlas_user_by_username(&self, org_id: &str, username: &str) -> Result<Option<UserResponse>> {
        let url = format!("{}/orgs/{}/users", ATLAS_API_V2_BASE_URL, org_id);

        let response = self
            .client
            .get(&url)
            .bearer_auth(self.access_token.as_ref())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let users: UsersListResponse = handle_ok_response(response).await?;
                Ok(users.results.into_iter().find(|u| u.username == username))
            }
            status => handle_error(status, response).await,
        }
    }
}

/// Response wrapper for listing users
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct UsersListResponse {
    results: Vec<UserResponse>,
}

async fn handle_ok_response<A>(response: Response) -> Result<A>
where
    A: for<'de> Deserialize<'de>,
{
    let content = response.bytes().await?;
    let parsed = serde_json::from_slice(&content)?;
    Ok(parsed)
}

async fn handle_error<A>(status: StatusCode, response: Response) -> Result<A> {
    let message = response.text().await?;
    Err(Error::Api { status, message })
}
