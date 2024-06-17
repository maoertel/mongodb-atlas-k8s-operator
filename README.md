# MongoDB Atlas K8s Operator

As the orginal MongoDB Atlas K8s Operator is not taking Atlas users into account, I thought about writing my own
operator that handles this. And as there is already a lot of stuff out there for Golang, I decided to write it in Rust.

It is in a POC state so a lot of happy path and things that are not handled yet.

## Use it

1. Create the CRD

```bash
kubectl create -f crds/atlasusers.yaml
```

2. Create the operator

```bash
cargo run --public-key <public-key> --private-key <private>
```

3. Create a new MongoDB Atlas `AtlasUser` K8s resource

```bash
kubectl create -f crds/examples/john_doe.yaml
```

4. Check the created resource in you cluster

```bash
kubectl describe atlasusers johndoe
```
