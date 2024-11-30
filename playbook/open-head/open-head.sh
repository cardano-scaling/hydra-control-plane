#!/bin/sh
echo "---"
echo ""
echo "Getting UTxOs to generate init and commit..."
echo ""
echo "---"
SEED_INPUT=$(
  curl \
    -H "project_id: $BLOCKFROST_KEY" \
    "https://cardano-preprod.blockfrost.io/api/v0/addresses/$ADDRESS/utxos" \
    | jq -r '.[0] | "\(.tx_hash)#\(.tx_index)"' 
)
COMMIT_INPUT=$(
  curl \
    -H "project_id: $BLOCKFROST_KEY" \
    "https://cardano-preprod.blockfrost.io/api/v0/addresses/$ADDRESS/utxos" \
    | jq -r '.[-1] | "\(.tx_hash)#\(.tx_index)"' 
)

echo "* Seed input: $SEED_INPUT"
echo "* Commit input: $COMMIT_INPUT"

echo "---"
echo ""
echo "Opening head..."
echo ""
echo "---"
# Open head
cargo run --bin open-head -- \
  --network-id 0 \
  --seed-input $SEED_INPUT \
  --participant $ADDRESS \
  --party-verification-file $HYDRA_VERIFICATION_KEY_FILE \
  --cardano-key-file $ADMIN_SIGNING_KEY_FILE \
  --blockfrost-key $BLOCKFROST_KEY \
  --commit-inputs $COMMIT_INPUT

sleep 300

