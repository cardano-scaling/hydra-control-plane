 #!/bin/sh
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
 echo "Opening heads..."
 echo ""
 echo "---"
 # Open head
 for i in {1..5} ; do
    echo "LOOP $i"
     # Generate ID and keys, and persist
     NODE_ID=$(tr -dc 'a-z' < /dev/urandom | head -c 5)
     mkdir -p "/var/data/$NODE_ID/keys"
     /hydra-node gen-hydra-key --output-file "/var/data/$NODE_ID/keys/hydra"

     # Save starting point to reduce syncing time
     echo $(
         curl \
         -H "project_id: $BLOCKFROST_KEY" \
         "https://cardano-preprod.blockfrost.io/api/v0/blocks/latest" \
         | jq -r '"\(.slot).\(.hash)"'
     ) > "/var/data/$NODE_ID/start"

     LOGS=$(/hcp/open-head \
         --network-id 0 \
         --seed-input $SEED_INPUT \
         --participant $ADDRESS \
         --party-verification-file /var/data/$NODE_ID/keys/hydra.vk \
         --cardano-key-file $ADMIN_SIGNING_KEY_FILE \
         --blockfrost-key $BLOCKFROST_KEY \
         --commit-inputs $COMMIT_INPUT)

     if [ $? -eq 0 ]; then
         echo $LOGS
         TX_ID=$(echo $LOGS | tail -n 1)
         echo $TX_ID
         SEED_INPUT="$TX_ID#1"
         COMMIT_INPUT="$TX_ID#2"
     else
         echo "Error in transaction ${i}"
         echo $LOGS
         exit 1
     fi
     echo "waiting 10 seconds..."
     sleep 10
 done
