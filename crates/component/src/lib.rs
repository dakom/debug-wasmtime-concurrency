wit_bindgen::generate!({
    world: "my-world",
});

struct Component;

impl Guest for Component {
    fn run(loop_time_ms: u64) -> String {
        let start = std::time::Instant::now();
        let expires = std::time::Duration::from_millis(loop_time_ms);
        while start.elapsed() < expires {
            // busy wait
        }

        format!("looped for {}ms", loop_time_ms)
    }
}

export!(Component);
