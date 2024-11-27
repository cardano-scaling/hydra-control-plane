#!/bin/sh
if [[ -z "${KEY}" ]]; then
  echo "Snapshot does not exist, generating keys..."
  mkdir "$DATA_DIR/keys"
  /hydra-node gen-hydra-key --output-file "$DATA_DIR/keys/hydra"
else
  echo "Downloading snashot..."
  aws s3 cp "s3://$BUCKET/$KEY" "$DATA_DIR/snapshot.tar.gz"
  tar -xzvf "$DATA_DIR/snapshot.tar.gz" -C "$DATA_DIR"
fi
