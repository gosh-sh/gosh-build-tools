pub fn init() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .event_format(
            tracing_subscriber::fmt::format()
                .without_time()
                .with_ansi(true)
                .with_source_location(false),
        )
        .init();
}
