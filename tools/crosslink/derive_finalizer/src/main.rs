use std::io::Read;

fn main() {
    let path = std::env::args().nth(1).expect("seed path");
    let mut seed_bytes = [0u8; 32];
    std::fs::File::open(path)
        .unwrap()
        .read_exact(&mut seed_bytes)
        .unwrap();
    let bft_key_seed = format!("adrheardhed{:?}", seed_bytes);
    let (_rng, _sk, pk) = zebra_crosslink::rng_private_public_key_from_address(bft_key_seed.as_bytes());
    let mut bytes = pk.0;
    bytes.reverse();
    println!("{}", hex::encode(bytes));
}
