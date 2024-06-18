# MongoDB Atlas K8s Operator

As the orginal MongoDB Atlas K8s Operator is not taking Atlas users into account, I thought about writing my own
operator that handles this. And as there is already a lot of stuff out there for Golang, I decided to write it in Rust.

It is in a POC state so a lot of happy path and things that are not handled yet, like:

- What happens to the passwords (send encrypted via email, put into Vault?)
- All the error handling regarding Atlas API
- Update the Status of the k8s resource

## Use it

1. Create the CRD

```bash
kubectl create -f crds/atlasusers.yaml
```

2. Start the operator

In the context of your choice, start the operator with the following command. You need to provide atlas MongoDB API key credentials.

```bash
cargo run --public-key <public-key> --private-key <private>
```

3. Create a new MongoDB Atlas `AtlasUser` K8s resource

```bash
kubectl create -f crds/examples/john_doe.yaml
```

What basically deployes something like that:

```yaml
apiVersion: moertel.com/v1
kind: AtlasUser
metadata:
  name: johndoe
  namespace: default
spec:
  country: US
  firstName: John
  lastName: Doe
  username: johndoe@example.com
  roles:
  - orgId: "4723423423"
    roleName: "ORG_OWNER"
```

4. Check the created resource in your cluster

```bash
kubectl describe atlasusers johndoe
```
