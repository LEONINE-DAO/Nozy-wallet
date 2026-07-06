//! Ironwood lightwalletd smoke: GetLightdInfo + GetBlockRange near NU6.3 activation.

use zeaking::lwd::proto::{BlockId, BlockRange, Empty};

const DEFAULT_ACTIVATION: u64 = 4_134_000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let url = args
        .next()
        .unwrap_or_else(|| "http://127.0.0.1:9068".to_string());
    let start: u64 = args
        .next()
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(DEFAULT_ACTIVATION);
    let end: u64 = args
        .next()
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(start.saturating_add(2));

    if end < start {
        return Err("end height must be >= start height".into());
    }

    let mut client = zeaking::lwd::connect_lightwalletd(&url).await?;

    let info = client.get_lightd_info(Empty {}).await?.into_inner();
    println!("lightwalletd_url: {url}");
    println!("version: {}", info.version);
    println!("chain_name: {}", info.chain_name);
    println!("block_height: {}", info.block_height);
    if !info.zcashd_subversion.is_empty() {
        println!("backend: {}", info.zcashd_subversion);
    }

    let tip = info.block_height;
    if tip < start {
        eprintln!(
            "warn: chain tip {tip} is below scan start {start}; range may be empty or partial"
        );
    }

    let range = BlockRange {
        start: Some(BlockId {
            height: start,
            hash: vec![],
        }),
        end: Some(BlockId {
            height: end.min(tip),
            hash: vec![],
        }),
    };

    println!(
        "GetBlockRange: {start}..={}",
        range.end.as_ref().unwrap().height
    );

    let mut stream = client.get_block_range(range).await?.into_inner();
    let mut blocks = 0u64;
    let mut orchard_actions = 0usize;
    let mut ironwood_actions = 0usize;
    let mut max_ironwood_tree = 0u32;

    while let Some(block) = stream.message().await? {
        blocks += 1;
        if let Some(meta) = &block.chain_metadata {
            max_ironwood_tree = max_ironwood_tree.max(meta.ironwood_commitment_tree_size);
        }
        for tx in &block.vtx {
            orchard_actions += tx.actions.len();
            ironwood_actions += tx.ironwood_actions.len();
        }
        println!(
            "  block {} vtx={} orchard_actions={} ironwood_actions={} ironwood_tree={}",
            block.height,
            block.vtx.len(),
            block.vtx.iter().map(|t| t.actions.len()).sum::<usize>(),
            block
                .vtx
                .iter()
                .map(|t| t.ironwood_actions.len())
                .sum::<usize>(),
            block
                .chain_metadata
                .as_ref()
                .map(|m| m.ironwood_commitment_tree_size)
                .unwrap_or(0)
        );
    }

    println!("summary: blocks={blocks} orchard_actions={orchard_actions} ironwood_actions={ironwood_actions} max_ironwood_tree={max_ironwood_tree}");

    if blocks == 0 {
        eprintln!("FAIL: GetBlockRange returned no blocks (is lightwalletd synced past {start}?)");
        std::process::exit(1);
    }

    if start >= DEFAULT_ACTIVATION && ironwood_actions == 0 && max_ironwood_tree == 0 {
        eprintln!(
            "WARN: no ironwoodActions or ironwoodCommitmentTreeSize in range {start}..={end} \
             (activation blocks may be empty; retry nearer tip, e.g. 4136100)"
        );
        std::process::exit(2);
    }

    println!("OK: Ironwood LWD GetBlockRange smoke passed");
    Ok(())
}
