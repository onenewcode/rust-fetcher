fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc_path = protoc_bin_vendored::protoc_bin_path()?;

    println!("cargo:rerun-if-changed=proto/douyin.proto");
    println!("cargo:rerun-if-changed=build.rs");

    let mut config = prost_build::Config::new();
    config.protoc_executable(protoc_path);
    config.compile_protos(&["proto/douyin.proto"], &["proto"])?;

    Ok(())
}
