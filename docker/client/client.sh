#!/usr/bin/env bash
set -e

echo "Server addr: $SERVER_IP:$SERVER_PORT"

# Retry loop
while ! nc -z -w 5 $SERVER_IP $SERVER_PORT; do
  echo "Waiting for server $SERVER_IP:$SERVER_PORT ... "
  sleep 1
done

cargo run --release --bin grpc-zk-example-client -- --server=http://$SERVER_IP:$SERVER_PORT