fn main() {
    tonic_build::configure()
        .compile(
            &["proto/gosh-get.proto", "proto/git-remote-gosh.proto"],
            &["proto"],
        )
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });
}
