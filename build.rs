// build.rs

use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Obtain the path to the vendored `protoc` binary
    let protoc = protoc_bin_vendored::protoc_bin_path()?;

    // Set the `PROTOC` environment variable to the full path of `protoc.exe`
    env::set_var("PROTOC", protoc.to_str().unwrap());

    // Initialize prost_build configuration
    let prost_config = prost_build::Config::new();

    // Configure and compile the protobuf definitions
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_with_config(prost_config, &["src/proto/cache.proto"], &["src/proto"])?;

    Ok(())
}
