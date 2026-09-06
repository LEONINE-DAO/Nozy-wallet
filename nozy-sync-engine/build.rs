//! Compile lightwalletd CompactTxStreamer protos (client unused; server + types).

fn main() {
    println!("cargo:rerun-if-changed=proto/service.proto");
    println!("cargo:rerun-if-changed=proto/compact_formats.proto");

    let proto_dir = std::path::PathBuf::from("proto");
    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(
            &[proto_dir.join("service.proto")],
            std::slice::from_ref(&proto_dir),
        )
        .expect("compile lightwalletd protos for nozy-sync-engine");
}
