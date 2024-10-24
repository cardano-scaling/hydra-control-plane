# Hydra Control Plane for Hydra Doom

Orchestrating API server that provides "Managed Hydra Head" instance access to players of [hydra-doom](https://github.com/cardano-scaling/hydra-doom).

## Getting started

First, we need to generate an admin key that manages state in Hydra heads:

``` sh
cardano-cli address key-gen --normal-key --verification-key-file admin.vk --signing-key-file admin.sk
```

If you don't want to run it locally on port 8000, reconfigure by amending `Rocket.toml`.

Next, we need to ensure the control plane can reach a locally running
`hydra-node`. To get started quickly, we'll prepare an `offline` mode head which is
directly open and has funds owned by the generated admin key:

``` sh
hydra-node gen-hydra-key --output-file hydra

curl https://raw.githubusercontent.com/cardano-scaling/hydra/0.17.0/hydra-cluster/config/protocol-parameters.json \
  | jq '.utxoCostPerByte = 0' > protocol-parameters.json

cat > utxo.json << EOF
{
  "0000000000000000000000000000000000000000000000000000000000000000#0": {
    "address": "$(cardano-cli address build --verification-key-file admin.vk --testnet-magic 1)",
    "value": {
      "lovelace": 1000000000
    }
  }
}
EOF
```

To start the `hydra-node`:

``` sh
hydra-node offline \
  --hydra-signing-key hydra.sk \
  --ledger-protocol-parameters protocol-parameters.json \
  --initial-utxo utxo.json \
  --persistence-dir hydra-state
```

Then, in a dedicated terminal, build & start with:

``` sh
cargo run --release
```

## Rocket.toml

You can configure the server in the Rocket.toml.

Each `[abc]` section defines a "Profile", which you can switch to by setting the ROCKET_PROFILE environment variable. The default is `[default]`.

### Nodes

You can configure remote hydra nodes with a `[[profile.nodes]]` entry, which can be repeated any number of times.

Each node has a `local_url`, which is the URL the control plane will attempt to connect on, and a `remote_url`, which is the URL the control plane will direct others to connect on. Don't include the port here.

`port` is the port to connect on

`admin_key_file` must point to the admin key generated above.

`persisted` means the node is persisting events to disk, and is reserved for the on-site cabinets, while remote players will be directed to non-persistent nodes.

`reserved` means the node will only be assigned games that pass the `?reserved` flag

`region` means the aws region the node is in, to give preference for people who ask for `?region=`

`stats_file` is an optional file where stats for this node should be persisted

`max_players` determines the maximum number of players that can be assigned to this node at once

### Hosts

You can configure nodes in bulk by configuring `[[profile.hosts]]` instead.

It has most of the same flags, except:

`port` is replaced by `start_port` and `end_port`;
`stats_file` is replaced by `stats_file_prefix`; the port will be added to this prefix to determine the actual file

All other flags get copied to the Node config.


## Open Head Binary
The `open-head` binary will build and submit the transactions necessary to open a hydra head with the specified arguments. Currently, it only supports one participant per head.

Example:
```
cargo run --bin open-head -- --seed-input e9a81012c52f175287ca2f0b73912915f6a75aa0d21b339e9af0af707674d0ad#2 \
--cardano-key-file preprod.sk \
--blockfrost-key [redacted] \
--party 7bbfc8ffc6da9e6f6f070f0f28a4c0de8e099c34485e192660475059d8bb9557 \
--participant addr_test1vzdjnh24kw99aqj8whfsxu37s0sgmq7yhfeva2egg92t3gsws2hwn \
--commit-inputs fee65a89c2f26958bceb29233ef5cc9d5ad20b67f55150bdc38711e7cff4e0fa#0
```
