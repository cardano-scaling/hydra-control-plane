# Hydra Control Plane for Hydra Doom

Orchestrating API server that provides "Managed Hydra Head" instance access to players of [hydra-doom](https://github.com/cardano-scaling/hydra-doom).

## Getting started

First, we need to generate an admin key that manages state in Hydra heads:

``` sh
cardano-cli address key-gen --normal-key --verification-key-file admin.vk --signing-key-file admin.sk
```

If you don't want to run it locally on port 8000, reconfigure by amending `Rocket.yaml`.

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
