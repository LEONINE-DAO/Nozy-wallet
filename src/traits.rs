use crate::error::NozyResult;

/// Key derivation abstraction.
///
/// Current implementation: Orchard (BIP39 -> Orchard SpendingKey -> FullViewingKey)
/// Future: Tachyon simplified key structure (backward-compatible)
pub trait KeyDerivation {
    type SpendingKey;
    type FullViewingKey;
    type IncomingViewingKey;

    fn derive_spending_key(&self, seed: &[u8], account: u32) -> NozyResult<Self::SpendingKey>;
    fn derive_full_viewing_key(&self, sk: &Self::SpendingKey) -> Self::FullViewingKey;
    fn derive_incoming_viewing_key(&self, fvk: &Self::FullViewingKey) -> Self::IncomingViewingKey;
    fn default_address(&self, fvk: &Self::FullViewingKey) -> NozyResult<String>;
}

/// Decrypted note from scanning a block.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecryptedNote {
    pub value: u64,
    pub nullifier: Vec<u8>,
    pub block_height: u32,
    pub txid: String,
    pub address: String,
    pub memo: Vec<u8>,
}

/// Note scanning abstraction.
///
/// Current implementation: Orchard compact note decryption using IVK
/// Future: Tachyon oblivious synchronization (nullifiers evolve in an unlinkable way)
pub trait NoteScanning {
    type ViewingKey;

    fn scan_block(
        &self,
        ivk: &Self::ViewingKey,
        block_data: &[u8],
        block_height: u32,
    ) -> NozyResult<Vec<DecryptedNote>>;

    fn compute_nullifier(
        &self,
        note: &DecryptedNote,
        key: &[u8],
    ) -> NozyResult<Vec<u8>>;
}

/// Output description for building a transaction.
#[derive(Debug, Clone)]
pub struct OutputSpec {
    pub recipient: String,
    pub amount: u64,
    pub memo: Vec<u8>,
}

/// Result of proving a transaction.
#[derive(Debug, Clone)]
pub struct ProvedTransaction {
    pub raw_tx: Vec<u8>,
    pub txid: String,
}

/// Transaction proving abstraction.
///
/// Current implementation: Orchard Halo2 proving
/// Future: Tachyon proof-carrying data (PCD) with transaction aggregation
pub trait TransactionProving {
    type SpendableNote;
    type ProvingKey;

    fn build_and_prove(
        &self,
        spending_key: &[u8],
        inputs: &[Self::SpendableNote],
        outputs: &[OutputSpec],
        proving_key: &Self::ProvingKey,
    ) -> NozyResult<ProvedTransaction>;

    fn estimate_proving_time_ms(
        &self,
        num_inputs: usize,
        num_outputs: usize,
    ) -> u64;
}

/// Storage abstraction for platform-independent persistence.
///
/// Native: uses std::fs on the local filesystem
/// WASM: uses chrome.storage.local via JS callbacks
pub trait StorageBackend {
    fn read(&self, key: &str) -> NozyResult<Option<Vec<u8>>>;
    fn write(&self, key: &str, data: &[u8]) -> NozyResult<()>;
    fn exists(&self, key: &str) -> NozyResult<bool>;
    fn delete(&self, key: &str) -> NozyResult<()>;
    fn list_keys(&self, prefix: &str) -> NozyResult<Vec<String>>;
}
