#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -ex

cd /opt/starcoin/etc 
echo "$NODE_CONFIG" > node.config.toml 
echo "$SEED_PEERS" > seed_peers.config.toml 
echo "$NETWORK_KEYPAIRS" > network_keypairs.config.toml 
echo "$NETWORK_PEERS" > network_peers.config.toml 
echo "$CONSENSUS_KEYPAIR" > consensus_keypair.config.toml 
echo "$CONSENSUS_PEERS" > consensus_peers.config.toml 
echo "$FULLNODE_KEYPAIRS" > fullnode_keypairs.config.toml 
exec /opt/starcoin/bin/sgchain -f node.config.toml
