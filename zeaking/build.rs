//! gRPC stubs for `lightwalletd` (optional feature).

fn main() {
    println!("cargo:rerun-if-changed=proto/service.proto");
    println!("cargo:rerun-if-changed=proto/compact_formats.proto");

    if std::env::var("CARGO_FEATURE_LIGHTWALLETD").is_err() {
        return;
    }

    let proto_dir = std::path::PathBuf::from("proto");
    tonic_prost_build::configure()
        .build_client(true)
        .build_server(false)
        .compile_protos(
            &[proto_dir.join("service.proto")],
            std::slice::from_ref(&proto_dir),
        )
        .expect("compile lightwalletd protos");
}
