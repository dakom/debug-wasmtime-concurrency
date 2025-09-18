# FIX

Thanks to Alex Crichton for [sharing the fix!](https://bytecodealliance.zulipchat.com/#narrow/channel/206238-general/topic/running.20arbitrary.20async.20components.20safely/near/540250536)

Committed in https://github.com/dakom/debug-wasmtime-concurrency/commit/6b6034decc688a73c4149afffd2f2a2bf965e044

In other words, use `epoch_deadline_callback` with an explicit `tokio::task::yield_now()` instead of `epoch_deadline_async_yield_and_update()`

### Build the component

```bash
cargo build --package example --target wasm32-wasip2
```

# Testing

```bash
cargo test --package app
```

# Reproducing the error

First, make sure you're on the [broken](https://github.com/dakom/debug-wasmtime-concurrency/tree/broken) branch.

Change [YIELD_PERIOD_MS](./crates/app/tests/concurrency.rs#L5) from 10 to 100 and run the tests again.

Even though it should be yielding every 100 ms or so, the test will hang for the full 5 seconds.

Yet setting it to 10 works, presumably because it yields often enough to let the "quick" task complete

# Alternative

There's a crossbeam-channel alternative in [alternative-crossbeam](https://github.com/dakom/debug-wasmtime-concurrency/tree/alternative-crossbeam) but the results are the same
