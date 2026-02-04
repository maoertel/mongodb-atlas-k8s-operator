use std::sync::Arc;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;

use crate::crd::AtlasUserRoles;
use crate::crd::UserOrgMembershipStatus;

/// Response from Atlas API for user operations
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    /// The Atlas user ID
    pub id: Arc<str>,
    /// The user's organization membership status
    pub org_membership_status: UserOrgMembershipStatus,
    /// The user's roles
    pub roles: AtlasUserRoles,
    /// Team IDs the user is assigned to
    #[serde(default)]
    pub team_ids: Vec<String>,
    /// The user's email address
    pub username: String,
    /// When the invitation was created (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invitation_created_at: Option<DateTime<Utc>>,
    /// When the invitation expires (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invitation_expires_at: Option<DateTime<Utc>>,
    /// User's country (filled by user after accepting invite)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    /// User's first name (filled by user after accepting invite)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    /// User's last name (filled by user after accepting invite)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
}
