use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub country: String,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub roles: Option<Vec<Role>>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    pub group_id: Option<String>,
    pub org_id: Option<String>,
    pub role_name: String,
}
