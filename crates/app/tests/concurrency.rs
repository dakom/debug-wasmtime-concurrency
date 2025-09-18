use app::{App, ExecutionConfig};
use std::{sync::LazyLock, time::Duration};
use tokio::task::spawn_blocking;

// CHANGE TO 100 TO SEE THE PROBLEM
const YIELD_PERIOD_MS: u64 = 10;

// NO NEED TO CHANGE ANYTHING ELSE BELOW
const COMPONENT_PATH: &str = "../../target/wasm32-wasip2/debug/example.wasm";

static COMPONENT_BYTES: LazyLock<Vec<u8>> = LazyLock::new(|| {
    std::fs::read(COMPONENT_PATH).expect(&format!(
        "Failed to read component at {COMPONENT_PATH}, see the README"
    ))
});

#[tokio::test]
async fn sanity_check() {
    let app = App::new();

    let res = app
        .execute(
            &COMPONENT_BYTES,
            ExecutionConfig {
                loop_time_ms: 10,
                yield_period_ms: YIELD_PERIOD_MS,
            },
        )
        .await;
    assert_eq!(res, "looped for 10ms");
}

#[tokio::test(flavor = "current_thread")]
async fn should_not_block() {
    let app = App::new();

    // try to tie up the runtime
    let (slow_tx, _) = crossbeam::channel::unbounded();
    tokio::spawn({
        let app = app.clone();
        async move {
            let res = app
                .execute(
                    &COMPONENT_BYTES,
                    ExecutionConfig {
                        loop_time_ms: 10_000,
                        yield_period_ms: YIELD_PERIOD_MS,
                    },
                )
                .await;
            slow_tx.send(res).unwrap();
        }
    });

    // so that this quick task doesn't complete fast
    let (quick_tx, quick_rx) = crossbeam::channel::unbounded();
    tokio::spawn({
        let app = app.clone();
        async move {
            let res = app
                .execute(
                    &COMPONENT_BYTES,
                    ExecutionConfig {
                        loop_time_ms: 10,
                        yield_period_ms: YIELD_PERIOD_MS,
                    },
                )
                .await;

            quick_tx.send(res).unwrap();
        }
    });

    let time = std::time::Instant::now();
    let res = spawn_blocking(move || quick_rx.recv().unwrap())
        .await
        .unwrap();

    assert_eq!(res, "looped for 10ms");
    if time.elapsed() >= Duration::from_secs(5) {
        panic!("took way too long for tasks to complete!");
    }

    println!("Took {}ms", time.elapsed().as_millis());
}
