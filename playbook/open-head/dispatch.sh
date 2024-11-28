if [[ -z "${BLOCKFROST_KEY}" ]]; then
  echo "Missing BLOCKFROST_KEY env var."
  exit 1
fi

if [[ -z "${DMTR_API_KEY}" ]]; then
  echo "Missing DMTR_API_KEY env var."
  exit 1
fi

ID=testonlinenewscript
HYDRA_NODE_IMAGE=ghcr.io/demeter-run/hydra-node:patch2
HYDRA_SCRIPTS_TX_ID=af8c6a99c26277621b36b8aa4dd97a2f03316bd9aae804ba26f17d7cbea85197
START_CHAIN_FROM=77141816.c7f5806dfd978474ab23941fcf2c552e7fca13749b7791fd0b3aa83dc1bcd173
PVC_NAME=open-head-volume

cat job.yml \
  | sed -E 's@\{id\}@'"$ID"'@g' \
  | sed -E 's@\{blockfrost_key\}@'"$BLOCKFROST_KEY"'@g' \
  | sed -E 's@\{hydra-node-image\}@'"$HYDRA_NODE_IMAGE"'@g' \
  | sed -E 's@\{hydra_scripts_tx_id\}@'"$HYDRA_SCRIPTS_TX_ID"'@g' \
  | sed -E 's@\{start_chain_from\}@'"$START_CHAIN_FROM"'@g' \
  | sed -E 's@\{dmtr_api_key\}@'"$DMTR_API_KEY"'@g' \
  | sed -E 's@\{pvc_name\}@'"$PVC_NAME"'@g' \
  | kubectl apply -f -
