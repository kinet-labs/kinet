#!/bin/bash

# set environment variables
KINET_ROOT=$(git rev-parse --show-toplevel)
export KINET_ROOT
export KINET_EXECUTION_ROOT="${KINET_ROOT}/kinet-execution"
export HOST_UID="${HOST_UID:-$(id -u)}"
export HOST_GID="${HOST_GID:-$(id -g)}"
DOCKER_USER="${HOST_UID}:${HOST_GID}"

# clean up environment
cd $KINET_ROOT/docker/devnet
bash clean.sh
cd $KINET_ROOT

# build kinet execution
set +e
docker buildx inspect insecure
insecure_builder_no_exist=$?
set -e
if [ $insecure_builder_no_exist -ne 0 ]; then
    docker buildx create --buildkitd-flags '--allow-insecure-entitlement security.insecure' --name insecure
fi
docker build --builder insecure --allow security.insecure \
    -f $KINET_EXECUTION_ROOT/docker/release.Dockerfile \
    --load -t kinet-execution:latest $KINET_EXECUTION_ROOT \
    --build-arg GIT_COMMIT_HASH=$(git -C $KINET_EXECUTION_ROOT rev-parse HEAD)

# build kinet consensus
docker build -t kinet-node:latest \
    -f $KINET_ROOT/docker/devnet/Dockerfile $KINET_ROOT

# build kinet rpc
docker build -t kinet-rpc:latest \
    -f $KINET_ROOT/docker/rpc/Dockerfile $KINET_ROOT

# start 1) execution -> 2) consensus -> 3) RPC server
mkdir -p $KINET_ROOT/docker/devnet/kinet/triedb
truncate -s 4GB $KINET_ROOT/docker/devnet/kinet/triedb/test.db
docker run \
    --user "$DOCKER_USER" \
    --security-opt seccomp:$KINET_ROOT/docker/devnet/kinet/config/profile.json \
    --volume $KINET_ROOT/docker/devnet/kinet:/kinet \
    kinet-execution:latest kinet_mpt --storage /kinet/triedb/test.db --create
echo "TrieDB has been initialized correctly."

EXECUTION_CONTAINER_ID=$(docker run \
    -d --user "$DOCKER_USER" \
    --security-opt seccomp:$KINET_ROOT/docker/devnet/kinet/config/profile.json \
    --volume $KINET_ROOT/docker/devnet/kinet:/kinet \
    kinet-execution:latest kinet --chain kinet_devnet --db /kinet/triedb/test.db --block_db /kinet/ledger)
echo "Execution node has been started."

CONSENSUS_CONTAINER_ID=$(docker run \
    -d --user "$DOCKER_USER" --volume $KINET_ROOT/docker/devnet/kinet:/kinet \
    kinet-node:latest)
echo "Consensus node has been started."

RPC_CONTAINER_ID=$(docker run \
    -d --user "$DOCKER_USER" \
    --security-opt seccomp:$KINET_ROOT/docker/devnet/kinet/config/profile.json \
    -p 8080:8080 --volume $KINET_ROOT/docker/devnet/kinet:/kinet \
    kinet-rpc:latest)
echo "RPC server has been started."

# call to RPC endpoints
function continuous_curl() {
    local url="http://localhost:8080"
    local data='{"jsonrpc":"2.0", "method":"eth_blockNumber", "params":[],"id":1}'

    while true; do
        curl "$url" -X POST --data "$data" > /dev/null 2>&1 || true
        sleep 0.1 # Sleep for 1ms
    done
}

# start the rpc calls in the background
continuous_curl &
# capture the PID of the background job
CURL_PID=$!
echo "Started RPC calls."

# stop and restart RPC, continue sending RPC calls when it's down, make sure that it restarts successfully

# stop the RPC server container
docker stop $RPC_CONTAINER_ID
echo "RPC server has been stopped."
sleep 15

# restart the RPC server container
RPC_CONTAINER_ID=$(docker run \
    -d --user "$DOCKER_USER" \
    --security-opt seccomp:$KINET_ROOT/docker/devnet/kinet/config/profile.json \
    -p 8080:8080 --volume $KINET_ROOT/docker/devnet/kinet:/kinet \
    kinet-rpc:latest)
echo "RPC server has been restarted."
sleep 5

# cleanup function to stop docker containers
cleanup() {
    echo "Cleaning up..."
    docker stop $EXECUTION_CONTAINER_ID $CONSENSUS_CONTAINER_ID $RPC_CONTAINER_ID
    kill $CURL_PID
    echo "Stopped execution, consensus, and RPC Docker containers."
}
trap cleanup EXIT

# make sure RPC server is still running
CHAIN_ID=$(curl -X POST --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' http://localhost:8080 | jq -r '.result')
EXPECTED_CHAIN_ID="0xa1ee"
echo "Received chain ID: $CHAIN_ID"

# Assertion
if [ "$CHAIN_ID" == "$EXPECTED_CHAIN_ID" ]; then
    echo "RPC server is running and returned the expected chain ID: $CHAIN_ID"
else
    echo "Unexpected chain ID: $CHAIN_ID"
    exit 1
fi

echo "Test script completed."
