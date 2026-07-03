//! Operator check: lightwalletd gRPC GetLightdInfo (chain tip + donation UA for Zec.rocks rewards).

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "http://127.0.0.1:9067".to_string());

    let mut client = zeaking::lwd::connect_lightwalletd(&url).await?;
    use zeaking::lwd::proto::Empty;
    let info = client.get_lightd_info(Empty {}).await?.into_inner();

    println!("lightwalletd_url: {}", url);
    println!("version: {}", info.version);
    println!("chain_name: {}", info.chain_name);
    println!("block_height: {}", info.block_height);
    println!("estimated_height: {}", info.estimated_height);
    if !info.zcashd_subversion.is_empty() {
        println!("backend: {}", info.zcashd_subversion);
    }

    let donation = info.donation_address.trim();
    if donation.is_empty() {
        eprintln!("donation_address: MISSING (add --donation-address to lightwalletd for Zec.rocks rewards)");
        std::process::exit(2);
    }
    println!("donation_address: {}", donation);
    if !donation.starts_with("u1") {
        eprintln!(
            "warn: donation address should be a unified address (u1...) with Orchard receiver"
        );
    }
    Ok(())
}
