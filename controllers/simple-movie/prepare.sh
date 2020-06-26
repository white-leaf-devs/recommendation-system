#!/bin/bash 

command -v "mongo" || ( echo "!! 'mongo' not found in path, aborting" && exit 1 )
command -v "diesel" || ( echo "!! 'diesel' not found in path, aborting" && exit 1 )
command -v "cargo" || ( echo "!! 'cargo' not found in path, aborting" && exit 1 )

DIR="$(dirname "$(readlink -f "$0")")"
pushd "$DIR" &> /dev/null

echo "=> Preparing things for simple-movie!"
diesel setup

echo "=> Loading main data"
cargo run --release --bin load_data

echo "=> Loading means"
cargo run --release --bin load_means

echo "=> Loading users who rated"
cargo run --release --bin load_users_who_rated

echo "=> Creating indexes on relations"
diesel migration --migration-dir indexes run

source ".env"
MONGO_CONN="${MONGO_URL}/${MONGO_DB}"

echo "=> Creating indexes on mongodb"
mongo "${MONGO_CONN}" --eval "db.users_who_rated.createIndex({ 'item_id': 'hashed' })"

popd &> /dev/null
