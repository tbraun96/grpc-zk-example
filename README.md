# grpc-zk-example

### Entrypoint
You can use docker-compose to execute the server and client:

```bash
docker-compose up --abort-on-container-exit
```

The process will take the exit code from the first process to exit, which should ideally be the client after it
successfully performs the ZKP protocol with the server

Alternatively, you can run the server first:

```bash
cargo run --release --bin grpc-zk-example-server -- --bind=127.0.0.1:12345
```

Then, once the server loads, you can run the client:

```bash
cargo run --release --bin grpc-zk-example-client -- --server=http://127.0.0.1:12345
```

### Disclaimers

This is an experimental repository not intended for production use. A serious implementation would not have an in-memory database for persistence.