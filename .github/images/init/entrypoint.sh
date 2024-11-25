#!/bin/sh
if [[ -z "${URI}" ]]; then
  echo "Snapshot does not exist, generating keys..."
  mkdir "$DATA_DIR/keys"
  /hydra-node gen-hydra-key --output-file "$DATA_DIR/keys/hydra"
else
  echo "Downloading snashot..."
  aws s3 cp "s3://$BUCKET/$KEY" "$DATA_DIR"
  tar -xzvf "$DATA_DIR/$KEY" -C "$DATA_DIR"
fi
