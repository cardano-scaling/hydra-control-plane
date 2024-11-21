# Scripts

## CRD

The `crd.sh` is there to update the CRD of the HydraDoom resource on the
bootstrapping folder. It dumps the definition generated on the Rust code into
YAML format, that is later formatted into K8s by the `tfk8s` bin.


## Prepare online node

To prepare an online node you must have:

* Permission to list and upload into `hydradoomsnapshots` bucket.
* `cardano-cli` in your PATH.
* `hydra-node` in your PATH.
* `admin.sk` and `admin.vk` files.
* A Blockfrost API key.
* A Cardano node-socket (`dmtrctl ports tunnel`).

To run the script:

```sh
./prepare_online_node.sh \
  --admin-signing-key-file PATH_TO_ADMIN_SK \
  --admin-verification-key-file PATH_TO_ADMIN_VK \
  --blockfrost-key YOUR_BF_API_KEY \
  --node-id NODE_ID \
  --protocol-parameters ../playbook/doom-dev/protocol-parameters.json \
  --cardano-node-socket PATH_TO_YOUR_SOCKET 
```

The script will:

1. Query the network to get available UTxOs to use as seed and commit inputs.
2. Generate a hydra key pair to use.
3. Query the network to get current tip.
4. Run `open-head` binary with the corresponding parameters.
5. Start the hydra-node with persistence activated. This process will run until
   it is externally killed. The user is supposed to `ctrl-c` once the node has
   acknowledged the transaction and the head is open.
6. After the hydra-node process is terminated, a tar file with the keys and
   persistence is uploaded to S3. You should see a `node.yml` with the
   parameters needed to create this resource on K8s.
