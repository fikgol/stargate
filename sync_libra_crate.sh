#!/usr/bin/env bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

declare -a libra_crates=("types" "config" "common/build_helpers" "common/canonical_serialization" "common/failure_ext" "common/grpc_helpers" "common/grpcio-client" "common/logger" "common/metrics" "common/proptest_helpers" "common/proto_conv" "common/channel" "crypto/crypto" "crypto/crypto_derive" "storage/accumulator" "storage/state_view" "storage/scratchpad" "storage/jellyfish_merkle" "storage/libradb" "storage/schemadb" "storage/storage_proto" "language/vm" "language/bytecode_verifier" "language/compiler" "language/stdlib" "language/functional_tests" "language/e2e_tests" "language/transaction_builder")

echo "Update git submodule"
git submodule init
#git submodule update

git submodule foreach git pull origin master

## now loop through the above array
for crate in "${libra_crates[@]}"
do
  FROM="$DIR/libra/$crate/"
  TO="$DIR/$crate"
  echo "sync $FROM with $TO";
  rsync -avu "$FROM" "$TO"
done

pushd language/vm/vm_genesis/ || exit
echo "generate new genesis blob"
cargo run
popd || exit

git status