use std::io::Result;

fn main() -> Result<()> {
    tonic_build::compile_protos("proto/builder.proto")?;

    // or

    // tonic_build::configure()
    //     .build_client(true)
    //     .build_server(true)
    //     .compile(&["proto/builder.proto"], &["proto"])?;
    Ok(())
}
