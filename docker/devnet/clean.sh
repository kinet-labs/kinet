#!/bin/bash

bft_ledger="./kinet/bft-ledger"
block_payload="./kinet/block-payload"
ledger="./kinet/ledger"
mempool_sock="./kinet/mempool.sock"
controlpanel_sock="./kinet/controlpanel.sock"
triedb="./kinet/triedb"
wal="./kinet/wal"
forkpoint="./kinet/config/forkpoint.toml"

# Check if directories/files from previous run exist, delete them if they exist
if [ -d "$ledger" ]; then
    rm -r "$ledger"
fi

if [ -d "$bft_ledger" ]; then
    rm -r "$bft_ledger"
fi

if [ -d "$block_payload" ]; then
    rm -r "$block_payload"
fi

if [ -d "$triedb" ]; then
    rm -r "$triedb"
fi

if [ -S "$mempool_sock" ]; then
    rm "$mempool_sock"
fi

if [ -S "$controlpanel_sock" ]; then
    rm "$controlpanel_sock"
fi

rm -f ${wal}_*

if [ -f "$forkpoint" ]; then
    rm "$forkpoint"
fi
