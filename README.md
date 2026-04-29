# bbstore

A toy implementation of a shared, in-memory key-value store server, built in Rust with Tokio.

The core design idea is the [single-writer principle](https://mechanical-sympathy.blogspot.com/2011/09/single-writer-principle.html): each shard owns its data exclusively and is mutated only by one actor. External access is achieved with `mpsc` (multi-producer, single-consumer) channel to have no shared-memory locking. 

**Disclaimer**: just a learning project, use it at your own risk.

## Running `bbstore`

The repository provides a server, client library and a cli to interact with the server.

Start the server:
```
RUST_LOG=debug cargo run --bin server
```

In a different terminal window, you can use the cli. For example, for setting a key-value pair:
```
cargo run --bin bbcli SET key value
```

Run:
```
cargo run --bin bbcli -- --help
```
to see all available options. 


## Roadmap

Look at the GitHub [issue tracker][issues] to contribute directly, this serves as a rough outline of the ideas:
- [ ] persistent storage
- [ ] DELETE command
- [ ] Tokio tracing
- [ ] graceful shutdown
- [ ] more benchmarks
- [ ] propagate errors to the user
- [ ] distributed storage
