#!/bin/sh

set -e
# Function to display help message
function show_help() {
  echo "Usage: $0 "
  echo "  --admin-signing-key-file <file> \\"
  echo "  --admin-verification-key-file <file> \\"
  echo "  --blockfrost-key <key> \\"
  echo "  --node-id <str> \\"
  echo "  --protocol-parameters <file> \\"
  echo "  --cardano-node-socket <file>"
  echo
  echo "This expects for you to have cardano-cli and hydra-node in your PATH."
  echo
  echo "Arguments:"
  echo "  --admin-signing-key-file      Path to the admin signing key file."
  echo "  --admin-verification-key-file Path to the admin verification key file."
  echo "  --blockfrost-key              Blockfrost API key."
  echo "  --node-id                     ID to set to the node being prepared."
  echo "  --protocol-parameters         Protocol parameters file."
  echo "  --cardano-node-socket         Path to socket."
  echo
  echo "Options:"
  echo "  --help                        Display this help message"
}

# If no arguments are provided, display help
if [ "$#" -eq 0 ]; then
  show_help
  exit 1
fi

# Check for --help flag
if [ "$1" == "--help" ]; then
  show_help
  exit 0
fi

# Check if exactly 12 arguments are provided
if [ "$#" -ne 12 ]; then
  echo "Error: Incorrect number of arguments."
  show_help
  exit 1
fi

# Check AWS privileges to upload snapshot
if ! aws s3 ls s3://hydradoomsnapshots/ > /dev/null 2>&1; then
  echo "Error: Unable to access S3 bucket. Please check your AWS permissions."
  exit 1
fi

# Check hydra-node binary is on path
if ! hydra-node --help > /dev/null 2>&1; then
  echo "Error: hydra-node not found on PATH."
  exit 1
fi

# Check cardano-cli is on path
if ! cardano-cli --help > /dev/null 2>&1; then
  echo "Error: cardano-cli not found on PATH."
  exit 1
fi

# Parse arguments
while [ "$#" -gt 0 ]; do
  case "$1" in
    --admin-signing-key-file)
      ADMIN_SIGNING_KEY_FILE="$2"
      shift 2
      ;;
    --admin-verification-key-file)
      ADMIN_VERIFICATION_KEY_FILE="$2"
      shift 2
      ;;
    --blockfrost-key)
      BLOCKFROST_KEY="$2"
      shift 2
      ;;
    --node-id)
      NODE_ID="$2"
      shift 2
      ;;
    --protocol-parameters)
      PROTOCOL_PARAMETERS="$2"
      shift 2
      ;;
    --cardano-node-socket)
      CARDANO_NODE_SOCKET="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      show_help
      exit 1
      ;;
  esac
done

echo "---"
echo ""
echo "Getting UTxOs to generate init and commit..."
echo ""
echo "---"
ADDRESS=$(cardano-cli address build --verification-key-file "$ADMIN_VERIFICATION_KEY_FILE" --testnet-magic 1)
SEED_INPUT=$(cardano-cli conway query utxo --address $ADDRESS --output-json --testnet-magic 1 --socket-path $CARDANO_NODE_SOCKET | jq -r 'to_entries[0].key')
COMMIT_INPUT=$(cardano-cli conway query utxo --address $ADDRESS --output-json --testnet-magic 1 --socket-path $CARDANO_NODE_SOCKET | jq -r 'to_entries[-1].key')

echo "* Seed input: $SEED_INPUT"
echo "* Commit input: $COMMIT_INPUT"

# Generate key pair.
mkdir keys
hydra-node gen-hydra-key --output-file keys/hydra

# Get current SLOT.HASH
START_POINT=$(cardano-cli query tip --socket-path socket --testnet-magic 1 | jq -r '"\(.slot).\(.hash)"')

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
  --party-verification-file keys/hydra.vk \
  --cardano-key-file $ADMIN_SIGNING_KEY_FILE \
  --blockfrost-key $BLOCKFROST_KEY \
  --commit-inputs $COMMIT_INPUT

# Start hydra-node in the background.
echo "---"
echo ""
echo "Running hydra-node with persistence. You should terminate this process when the head is already opened."
echo ""
echo "---"

set +e
hydra-node \
  --node-id $NODE_ID \
  --persistence-dir persistence \
  --cardano-signing-key $ADMIN_SIGNING_KEY_FILE \
  --hydra-signing-key keys/hydra.sk \
  --hydra-scripts-tx-id f41e346809f765fb161f060b3e40fac318c361f1be29bd2b827d46d765195e93 \
  --ledger-protocol-parameters $PROTOCOL_PARAMETERS \
  --testnet-magic 1 \
  --node-socket $CARDANO_NODE_SOCKET \
  --api-port 4001 \
  --host 0.0.0.0 \
  --api-host 0.0.0.0 \
  --port 5001 \
  --start-chain-from $START_POINT
set -e

echo "---"
echo ""
echo "Uploading tar..."
echo ""
echo "---"
tar -czvf "$NODE_ID.tar.gz" persistence keys
aws s3 cp "$NODE_ID.tar.gz" s3://hydradoomsnapshots/

echo "To run online node, apply the following:"
echo "---"
FILE=$(cat <<EOF
apiVersion: hydra.doom/v1alpha1
kind: HydraDoomNode
metadata:
  name: $NODE_ID
  namespace: hydra-doom
spec:
  seedInput: $SEED_INPUT
  commitInputs:
  - $COMMIT_INPUT
  startChainFrom: $START_POINT
EOF
)
printf "%s\n" "$FILE"
printf "%s\n" "$FILE" > "$NODE_ID.yml"

rm -rf persistence keys
