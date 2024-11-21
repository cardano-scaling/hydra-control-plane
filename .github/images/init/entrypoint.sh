#!/bin/sh
if aws s3 ls "s3://$BUCKET/$KEY" > /dev/null 2>&1; then
  echo "Snapshot exists, downloading..."
  aws s3 cp "s3://$BUCKET/$KEY" "$DATA_DIR"
  tar -xzvf "$DATA_DIR/$KEY" -C "$DATA_DIR"
else
  echo "Snapshot does not exist, generating keys..."
  mkdir "$DATA_DIR/keys"
  /hydra-node gen-hydra-key --output-file "$DATA_DIR/keys/hydra"
fi
