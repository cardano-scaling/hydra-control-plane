if [[ -z "${ID}" ]]; then
  echo "Missing ID env var."
  exit 1
fi

if [[ -z "${BLOCKFROST_KEY}" ]]; then
  echo "Missing BLOCKFROST_KEY env var."
  exit 1
fi

if [[ -z "${DMTR_API_KEY}" ]]; then
  echo "Missing DMTR_API_KEY env var."
  exit 1
fi

HYDRA_NODE_IMAGE=ghcr.io/demeter-run/hydra-node:patch2
HYDRA_SCRIPTS_TX_ID=f41e346809f765fb161f060b3e40fac318c361f1be29bd2b827d46d765195e93
START_CHAIN_FROM=77149305.7cff4a56346c7aa75b9f92bd4e860a0d9af2d83205f5c080fc94466cbe6054cc
PVC_NAME=open-head-volume

cat job.yml \
  | sed -E 's@\{id\}@'"$ID"'@g' \
  | sed -E 's@\{blockfrost_key\}@'"$BLOCKFROST_KEY"'@g' \
  | sed -E 's@\{hydra-node-image\}@'"$HYDRA_NODE_IMAGE"'@g' \
  | sed -E 's@\{hydra_scripts_tx_id\}@'"$HYDRA_SCRIPTS_TX_ID"'@g' \
  | sed -E 's@\{start_chain_from\}@'"$START_CHAIN_FROM"'@g' \
  | sed -E 's@\{dmtr_api_key\}@'"$DMTR_API_KEY"'@g' \
  | sed -E 's@\{ pvc_name \}@'"$PVC_NAME"'@g' \
  | kubectl apply -f -
