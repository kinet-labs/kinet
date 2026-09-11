# kinet-node

Starting a Kinet consensus node generates a blockdb directory, a ledger directory, a write ahead logging file, and an IPC socket:

Run the following in the repo root directory:
1. `export RUST_LOG=info`
    - The logging level can be adjusted as needed.
2. `cp docker/devnet/kinet/config/forkpoint.genesis.toml docker/devnet/kinet/config/forkpoint.toml`
    - Initialize consensus forkpoint to genesis
3. `CXX=/usr/bin/g++-15 CC=/usr/bin/gcc-15 ASMFLAGS=-march=haswell CFLAGS="-march=haswell" CXXFLAGS="-march=haswell" TRIEDB_TARGET=triedb_driver cargo run --bin kinet-node -- --secp-identity docker/devnet/kinet/config/id-secp --bls-identity docker/devnet/kinet/config/id-bls --node-config docker/devnet/kinet/config/node.toml --forkpoint-config docker/devnet/kinet/config/forkpoint.toml --wal-path docker/devnet/kinet/wal --mempool-ipc-path docker/devnet/kinet/mempool.sock --control-panel-ipc-path docker/devnet/kinet/controlpanel.sock --ledger-path docker/devnet/kinet/ledger --statesync-ipc-path docker/devnet/kinet/statesync.sock --triedb-path <path_to_triedb>`
    - The generated files and directories path (`--wal-path`, `--mempool-ipc-path`, `--control-panel-ipc-path`, `--ledger-path`, `--statesync-ipc-path`) can be changed.
