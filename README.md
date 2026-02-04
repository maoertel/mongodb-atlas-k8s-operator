# MongoDB Atlas K8s Operator

A Rust-based Kubernetes Operator for managing MongoDB Atlas users. Built with the [kuberator](https://crates.io/crates/kuberator) framework, this operator watches `AtlasUser` custom resources and manages corresponding users in MongoDB Atlas via the Atlas Admin API v2025-02-19 (invitation-based user management).

## Features

- Invite users to MongoDB Atlas organizations
- Manage organization and project-level role assignments
- Assign users to teams
- Track user status (Pending, Active, Deleted)
- Graceful shutdown handling (SIGINT/SIGTERM)
- Observed generation pattern for idempotent updates

## Prerequisites

- Kubernetes cluster
- MongoDB Atlas organization with API access
- Atlas OAuth access token (service account recommended)

## Installation

### 1. Apply the CRD

```bash
kubectl apply -f crds/atlasusers.yaml
```

### 2. Create a configuration file

```yaml
# config.yaml
atlas_user:
  requeue_duration: "1m"
  safe_to_delete: false
```

| Setting | Description |
|---------|-------------|
| `requeue_duration` | How often to requeue reconciliation |
| `safe_to_delete` | Whether to delete users from Atlas when the K8s resource is deleted |

### 3. Start the operator

```bash
cargo run -- --config config.yaml --access-token <your-atlas-oauth-token>
```

Or via environment variables:

```bash
export CONFIG_PATH=config.yaml
export ATLAS_ACCESS_TOKEN=<your-atlas-oauth-token>
cargo run
```

## Usage

### Create an AtlasUser

```bash
kubectl apply -f crds/examples/john_doe.yaml
```

Example resource:

```yaml
apiVersion: moertel.com/v1
kind: AtlasUser
metadata:
  name: john-doe
  namespace: default
spec:
  orgId: "your-org-id"
  username: "john.doe@example.com"
  roles:
    orgRoles:
      - ORG_MEMBER
    groupRoleAssignments:
      - groupId: "your-project-id"
        groupRoles:
          - GROUP_READ_ONLY
  teamIds: []
```

### Check the resource status

```bash
kubectl get atlasusers
kubectl describe atlasuser john-doe
```

### Available Roles

**Organization Roles:**
- `ORG_OWNER`
- `ORG_MEMBER`
- `ORG_GROUP_CREATOR`
- `ORG_BILLING_ADMIN`
- `ORG_BILLING_READ_ONLY`
- `ORG_READ_ONLY`

**Group (Project) Roles:**
- `GROUP_CLUSTER_MANAGER`
- `GROUP_DATA_ACCESS_ADMIN`
- `GROUP_DATA_ACCESS_READ_ONLY`
- `GROUP_DATA_ACCESS_READ_WRITE`
- `GROUP_OWNER`
- `GROUP_READ_ONLY`
- `GROUP_SEARCH_INDEX_EDITOR`
- `GROUP_STREAM_PROCESSING_OWNER`

## CLI Options

| Option | Environment Variable | Description |
|--------|---------------------|-------------|
| `--config`, `-c` | `CONFIG_PATH` | Path to configuration file (required) |
| `--access-token` | `ATLAS_ACCESS_TOKEN` | OAuth access token for Atlas API (required) |
| `--namespaces`, `-n` | - | Namespaces to watch (default: `default`) |

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Lint
cargo clippy

# Format
cargo fmt
```

## License

MIT
