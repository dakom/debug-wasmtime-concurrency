use wasmtime::component::bindgen;

bindgen!({
    world: "my-world",
    path: "../component/wit",
    imports: { default: async },
    exports: { default: async },
});
