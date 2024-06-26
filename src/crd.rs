use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// An `AtlasUser` struct is generated by the `CustomResource` derive macro.
/// This struct represents the spec part of the custom resource definition (CRD) for the `AtlasUser` resource.
#[derive(CustomResource, Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[kube(
    group = "moertel.com",
    version = "v1",
    kind = "AtlasUser",
    plural = "atlasusers",
    derive = "PartialEq",
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct AtlasUserSpec {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub country: String,
    pub roles: Vec<Role>,
    #[serde(default = "generate_password")]
    pub password: Option<String>,
    #[serde(default = "generate_dummy_mobile_number")]
    pub mobile_number: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    #[serde(skip_serializing_if = "Option::is_none")]
    org_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group_id: Option<String>,
    role_name: RoleName,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RoleName {
    OrgOwner,
    OrgMember,
    OrgGroupCreator,
    OrgBillingAdmin,
    OrgReadOnly,
    GroupClusterManager,
    GroupDataAccessAdmin,
    GroupDataAccessReadOnly,
    GroupDataAccessReadWrite,
    GroupOwner,
    GroupReadOnly,
    GroupSearchIndexEditor,
    GroupStreamProcessingOwner,
}

// TODO handle password creation etc properly through all layers
fn generate_password() -> Option<String> {
    Some("24#@$hlkjlj(^$@lljAA".to_string())
}

fn generate_dummy_mobile_number() -> Option<String> {
    Some("+49123456789".to_string())
}
