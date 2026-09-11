#!/bin/bash

set -ex

systemctl stop kinet kinet-execution kinet-rpc kinet-mpt kinet-execution-genesis || true

DB_MODE="slot"
MPT_OUTPUT=$(kinet-mpt --storage /dev/triedb 2>/dev/null || true)
if [ -z "$MPT_OUTPUT" ]; then
  echo "WARNING: kinet-mpt returned no output; defaulting to slot mode"
elif echo "$MPT_OUTPUT" | grep -q "Secondary:"; then
  DB_MODE="dual"
elif echo "$MPT_OUTPUT" | grep -q "State machine kind: kinet"; then
  DB_MODE="page"
fi
echo "Detected DB mode: $DB_MODE"
mkdir /home/kinet/kinet/empty-dir
rsync -r --delete /home/kinet/kinet/empty-dir/ /home/kinet/kinet/ledger/
rsync -r --delete /home/kinet/kinet/empty-dir/ /home/kinet/kinet/config/forkpoint/
rsync -r --delete /home/kinet/kinet/empty-dir/ /home/kinet/kinet/config/validators/
touch /home/kinet/kinet/ledger/wal
rm -rf /home/kinet/kinet/empty-dir
rm -rf /home/kinet/kinet/snapshots
rm -f /home/kinet/kinet/mempool.sock
rm -f /home/kinet/kinet/controlpanel.sock
rm -f /home/kinet/kinet/wal_*
rm -f /home/kinet/kinet/config/peers.toml
rm -rf /home/kinet/kinet/blockdb
source /home/kinet/.env
case "$DB_MODE" in
  dual)
    kinet-mpt --storage /dev/triedb --truncate --yes
    kinet-mpt --storage /dev/triedb --activate-secondary --state-machine kinet
    ;;
  page)
    kinet-mpt --storage /dev/triedb --truncate --state-machine kinet --yes
    ;;
  *)
    kinet-mpt --storage /dev/triedb --truncate --yes
    ;;
esac

if [ -f "/home/kinet/.config/forkpoint.genesis.toml" ]; then
  yes | cp -rf /home/kinet/.config/forkpoint.genesis.toml /home/kinet/kinet/config/forkpoint/forkpoint.toml
fi
if [ -f "/home/kinet/.config/validators.genesis.toml" ]; then
  yes | cp -rf /home/kinet/.config/validators.genesis.toml /home/kinet/kinet/config/validators/validators.toml
fi
