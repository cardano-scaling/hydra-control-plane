if [[ -z "${DMTR_API_KEY}" ]]; then
  echo "Missing DMTR_API_KEY env var."
  exit 1
fi

HYDRA_NODE_IMAGE=ghcr.io/demeter-run/hydra-node:patch2

cat job.yml \
  | sed -E 's@\{hydra-node-image\}@'"$HYDRA_NODE_IMAGE"'@g' \
  | sed -E 's@\{dmtr_api_key\}@'"$DMTR_API_KEY"'@g' \
  | kubectl apply -f -
