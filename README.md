# Prerequisites

### Build the component

```bash
cargo build --package example --target wasm32-wasip2
```

# Testing

```bash
cargo test --package app
```

# Reproducing the error

Change [YIELD_PERIOD_MS](./crates/app/tests/concurrency.rs#L5) from 10 to 100 and run the tests again.

Even though it should be yielding every 100 ms or so, the test will hang for the full 5 seconds.

Yet setting it to 10 works, presumably because it yields often enough to let the "quick" task complete
