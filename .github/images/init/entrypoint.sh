aws s3 cp "s3://$BUCKET/$KEY" "$DATA_DIR"
tar -xzvf "$DATA_DIR/$KEY" -C "$DATA_DIR"
