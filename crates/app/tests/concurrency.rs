use app::{App, ExecutionConfig};
use std::{sync::LazyLock, time::Duration};

// CHANGE TO 100 TO SEE THE PROBLEM
const YIELD_PERIOD_MS: u64 = 100;

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

#[test]
fn should_not_block() {
    let app = App::new();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // try to tie up the runtime
    let (slow_tx, _) = tokio::sync::oneshot::channel::<String>();
    rt.spawn({
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
    let (quick_tx, mut quick_rx) = tokio::sync::oneshot::channel::<String>();
    rt.spawn({
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

    rt.block_on(async move {
        let time = std::time::Instant::now();
        loop {
            match quick_rx.try_recv() {
                Ok(res) => {
                    assert_eq!(res, "looped for 10ms");
                    break;
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                    panic!("short task channel closed!");
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
            }

            tokio::task::yield_now().await;
        }

        if time.elapsed() >= Duration::from_secs(5) {
            panic!("took way too long for tasks to complete!");
        }

        println!("Took {}ms to complete", time.elapsed().as_millis());
    });
}
