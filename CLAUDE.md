# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust-based Kubernetes Operator for managing MongoDB Atlas users using the kuberator framework. The operator watches `AtlasUser` custom resources and manages corresponding users in MongoDB Atlas via the Atlas Admin API v2025-02-19 (invitation-based user management).

## Build and Development Commands

```bash
# Build the project
cargo build

# Run the operator (requires Atlas OAuth access token)
cargo run -- --access-token <token>
# Or via environment variables:
ATLAS_ACCESS_TOKEN=<token> cargo run

# With optional config file and namespace filtering:
cargo run -- --access-token <token> --config config.yaml --namespaces default --namespaces production

# Run tests
cargo test

# Format code (configured with max_width = 120)
cargo fmt

# Lint
cargo clippy

# Apply CRD to cluster
kubectl create -f crds/atlasusers.yaml

# Create example AtlasUser
kubectl create -f crds/examples/john_doe.yaml
```

## Architecture

### Reconciliation Flow (kuberator-based)

```
K8s Event → kuberator Controller → AtlasUserReconciler
    ↓
kuberator handles finalizers → Context::handle_apply() / handle_cleanup()
    ↓
AtlasUserContext → Atlas API calls + K8s status updates
```

### Key Modules

- **`src/operator/`** - `AtlasUserReconciler` implementing kuberator's `Reconcile` trait
- **`src/atlas/context.rs`** - Implements kuberator's `Context` trait with handle_apply/handle_cleanup
- **`src/atlas/repository.rs`** - Atlas API client with bearer token auth, handles user CRUD
- **`src/atlas/user_request.rs`** - Request DTOs for Atlas API
- **`src/atlas/user_response.rs`** - Response DTOs from Atlas API
- **`src/crd.rs`** - `AtlasUser` CRD definition with status (group: `moertel.com/v1`)
- **`src/k8s.rs`** - K8s repository type alias using kuberator's `K8sRepository`
- **`src/config.rs`** - Configuration loading from YAML

### Design Patterns

- **kuberator framework**: Uses `Reconcile`, `Context`, `Finalize`, and `ObserveGeneration` traits
- **Finalizer pattern**: Uses `atlasusers.moertel.com/finalizer` for safe deletion handling
- **Observed generation**: Status tracks `observed_generation` to detect spec changes
- **Invitation-based user management**: Atlas API v2025-02-19 uses invitations, not direct user creation
- **Module-level errors**: Each module has its own error type using `thiserror`

### CRD Spec Structure

```rust
AtlasUserSpec {
    org_id: String,           // Atlas organization ID
    username: String,         // Email address for invitation
    roles: AtlasUserRoles {
        org_roles: Vec<OrgRoleName>,
        group_role_assignments: Vec<GroupRoleAssignment>,
    },
    team_ids: Vec<String>,
}

AtlasUserStatus {
    user_id: Option<Arc<str>>,
    membership_status: Option<UserOrgMembershipStatus>,  // Active/Pending/Deleted
    observed_generation: Option<i64>,
    error: Option<String>,
}
```

### Configuration

```yaml
# config.yaml
atlas_user:
  requeue_duration: "1m"   # How often to requeue reconciliation
  safe_to_delete: false    # Whether to delete users from Atlas on K8s resource deletion
```

## Dependencies

- **kuberator v0.3.2**: Kubernetes operator framework
- **kube v2.0**: Kubernetes client
- **tracing**: Structured logging (JSON format)
- **reqwest**: HTTP client for Atlas API

## Known Limitations

- Uses OAuth bearer tokens (Atlas service accounts recommended for production)
- `safe_to_delete` defaults to false to prevent accidental user deletion
