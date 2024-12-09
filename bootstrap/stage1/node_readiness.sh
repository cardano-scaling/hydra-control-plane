#!/bin/bash

if ! command -v jq &> /dev/null; then
    echo "Error: jq is not installed. Please install jq to use this script."
    exit 1
fi

if ! command -v cardano-cli &> /dev/null; then
    echo "Error: cardano-cli is not installed. Please install cardano-cli to use this script."
    exit 1
fi

if ! command -v bc &> /dev/null; then
    echo "Error: bc is not installed. Please install bc to use this script."
    exit 1
fi

if [ "$CARDANO_NODE_NETWORK_ID" == "764824073" ]; then
    JSON_DATA=$(cardano-cli query tip --mainnet)
else
    JSON_DATA=$(cardano-cli query tip --testnet-magic "$CARDANO_NODE_NETWORK_ID")
fi

SYNC_PROGRESS=$(echo "$JSON_DATA" | jq -r '.syncProgress')
MIN_EXPECTED_SYNC_PROGRESS="99.00"
MAX_EXPECTED_SYNC_PROGRESS="100.00"

if (( $(echo "$SYNC_PROGRESS >= $MIN_EXPECTED_SYNC_PROGRESS" | bc -l) )) && (( $(echo "$SYNC_PROGRESS <= $MAX_EXPECTED_SYNC_PROGRESS" | bc -l) )); then
    echo "syncProgress is within the acceptable range of 99 to 100"
else
    echo "Error: syncProgress is not within the acceptable range of 99 to 100"
    exit 1
fi
