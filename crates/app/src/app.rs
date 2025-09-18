use std::time::Duration;

use wasmtime::{
    Config, Engine,
    component::{Component, Linker},
};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

#[derive(Clone)]
pub struct App {
    pub engine: Engine,
}

pub struct ExecutionConfig {
    pub loop_time_ms: u64,
    pub yield_period_ms: u64,
}

impl App {
    pub fn new() -> Self {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);
        config.consume_fuel(true);
        config.epoch_interruption(true);
        let engine = Engine::new(&config).unwrap();

        let engine_ticker = engine.weak();
        std::thread::spawn(move || {
            loop {
                if let Some(engine_ticker) = engine_ticker.upgrade() {
                    engine_ticker.increment_epoch();
                } else {
                    break;
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        });

        Self { engine }
    }

    pub async fn execute(
        &self,
        component_bytes: &[u8],
        ExecutionConfig {
            loop_time_ms,
            yield_period_ms,
        }: ExecutionConfig,
    ) -> String {
        let component = Component::new(&self.engine, component_bytes).unwrap();

        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::p2::add_to_linker_async(&mut linker).unwrap();

        let ctx = WasiCtxBuilder::new()
            .inherit_stdout()
            .inherit_stderr()
            .build();

        let state = State {
            ctx,
            table: wasmtime::component::ResourceTable::new(),
        };

        let mut store = wasmtime::Store::new(&self.engine, state);
        store.set_fuel(u64::MAX).unwrap();

        store.epoch_deadline_async_yield_and_update(yield_period_ms);

        crate::bindings::MyWorld::instantiate_async(&mut store, &component, &linker)
            .await
            .unwrap()
            .call_run(&mut store, loop_time_ms)
            .await
            .unwrap()
    }
}

struct State {
    pub ctx: WasiCtx,
    pub table: ResourceTable,
}

impl WasiView for State {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}
