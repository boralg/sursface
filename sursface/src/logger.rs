pub fn init_logger() {
    #[cfg(target_arch = "wasm32")]
    {
        let base_level = log::LevelFilter::Info;
        let wgpu_level = log::LevelFilter::Error;

        let _ = fern::Dispatch::new()
            .level(base_level)
            .level_for("wgpu_core", wgpu_level)
            .level_for("wgpu_hal", wgpu_level)
            .level_for("naga", wgpu_level)
            .chain(fern::Output::call(console_log::log))
            .apply();
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let base_level = log::LevelFilter::Info;
        let wgpu_level = log::LevelFilter::Error;

        // parse_default_env will read the RUST_LOG environment variable and apply it on top
        // of these default filters
        let _ = env_logger::builder()
            .filter_level(base_level)
            .filter_module("wgpu_core", wgpu_level)
            .filter_module("wgpu_hal", wgpu_level)
            .filter_module("naga", wgpu_level)
            .parse_default_env()
            .try_init();
    }
}
