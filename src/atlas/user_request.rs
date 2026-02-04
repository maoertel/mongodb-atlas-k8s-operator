use serde::Serialize;

use crate::crd::{AtlasUserRoles, AtlasUserSpec};

/// Request body for inviting or updating an Atlas user
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserRequest<'a> {
    /// Username (email) - only used for invite, not update
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<&'a str>,
    /// Role assignments
    pub roles: &'a AtlasUserRoles,
    /// Team IDs to assign
    pub team_ids: &'a [String],
}

impl<'a> UserRequest<'a> {
    /// Creates a new invite request (includes username)
    pub fn for_invite(spec: &'a AtlasUserSpec) -> Self {
        Self {
            username: Some(&spec.username),
            roles: &spec.roles,
            team_ids: &spec.team_ids,
        }
    }

    /// Creates an update request (excludes username)
    pub fn for_update(spec: &'a AtlasUserSpec) -> Self {
        Self {
            username: None,
            roles: &spec.roles,
            team_ids: &spec.team_ids,
        }
    }
}
