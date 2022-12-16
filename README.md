# lil-redis

> An intentionally-limited Rust implementation of the Redis server.

lil redis is an accessible implementation of a very basic Redis server (with `ping`, `echo`, `get`, and basic `set` support), with no external dependencies (other than `tokio` and `anyhow`).

## Motivation

I've been trying to get more serious about learning Rust lately, and [what better way to learn than by debugging](https://twitter.com/m1guelpf/status/1522100034875105282). So, when I discovered [CodeCrafters](https://codecrafters.io) (a platform that helps you get better at coding by guiding you through rebuilding popular tools, [referral link w/ discount](https://app.codecrafters.io/join?via=m1guelpf)), I decided to give their Redis guide a try.

Since the platform encourages you to come up with your own implementations, I've tried my best to make things as clean as possible (while hopefully keeping it simple enough for a beginner to understand). If you want to try your hand at it, I'd recommend [going through the guide first](https://app.codecrafters.io/join?via=m1guelpf), then comparing your solution to this one.

By sharing my implementation publicly, I hope to both attract others interested in learning Rust (who can use it as a learning resource) and already proficient with it (who can share which things they'd have done differently. PRs welcome!).

## Structure

The codebase is structured as follows:

```
lil-redis/
├─ src/
│ ├─ app.rs: TCP server, listening for connections, receiving and sending data.
│ ├─ cache.rs: Simple string store with built-in TTL support.
│ ├─ commands.rs: Redis command definition, parsing and logic.
│ ├─ main.rs: What gets called when you run the project
│ ├─ resp.rs: Encoding and decoding for Redis' RESP protocol
│ ├─ utils.rs: Helper functions
├─ tests/
│ ├─ redis_test.rs: Integration tests ensuring our server responds to requests by a Redis client
├─ Cargo.toml
├─ README.md
```

If you want to explore the codebase, I'd recommend starting with the `main.rs` file and going from there

> **Note** You can press `.` while on GitHub to launch a web VSCode instance, which should help you navigate the project better.

## Develop

Make sure you have the latest version of rust installed (use [rustup](https://rustup.rs/)). Then, you can build the project by running `cargo build`, run it with `cargo run`, and run the tests with `cargo test`.

## License

This project is open-sourced under the MIT license. See [the License file](LICENSE) for more information.
