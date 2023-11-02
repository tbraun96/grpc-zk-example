FROM rust:latest

WORKDIR /usr/src/server
COPY . .
RUN apt-get update && apt-get install -y protobuf-compiler
CMD cargo run --release --bin grpc-zk-example-server -- --bind=$BIND_ADDRESS