use crate::error::{NozyError, NozyResult}; 
use crate::notes::SpendableNote; 
use crate::zebra_integration::ZebraClient;
use orchard::{
    builder::Builder as OrchardBuilder,
    builder::BundleType,
    note::Note as OrchardNote,
    value::NoteValue,
    keys::{SpendingKey, FullViewingKey},
    bundle::Bundle,
    tree::{MerklePath, MerkleHashOrchard},
    Address as OrchardAddress,
    tree::Anchor,
    note::ExtractedNoteCommitment,
    bundle::Flags,
    builder::{InProgress, Unproven, Unauthorized, BundleMetadata, BuildError},
};
use zcash_address::unified::{Encoding, Container};
use ark_serialize::CanonicalSerialize;

use rand::{rngs::OsRng, Rng};

pub struct OrchardTransactionBuilder {
    proving_keys: Option<()>,
}

impl OrchardTransactionBuilder {
    pub fn new(_download_params: bool) -> Self {
        Self {
            proving_keys: None,
        }
    }

    pub async fn build_single_spend(
        &self,
        zebra_client: &ZebraClient,
        spendable_notes: &[SpendableNote],
        recipient_address: &str,
        amount_zatoshis: u64,
        fee_zatoshis: u64,
        memo: Option<&[u8]>,
    ) -> NozyResult<Vec<u8>> {
        println!("Building Orchard transaction...");

        let (_, recipient) = zcash_address::unified::Address::decode(recipient_address)
            .map_err(|e| NozyError::InvalidOperation(format!("Invalid recipient address: {}", e)))?;

        let total_input_value: u64 = spendable_notes.iter()
            .map(|note| note.orchard_note.value)
            .sum();

        let change_amount = total_input_value.saturating_sub(amount_zatoshis + fee_zatoshis);

        if total_input_value < amount_zatoshis + fee_zatoshis {
            return Err(NozyError::InvalidOperation(
                format!("Insufficient funds: have {} zatoshis, need {} zatoshis",
                    total_input_value, amount_zatoshis + fee_zatoshis)
            ));
        }

        let bundle_type = BundleType::Transactional {
            flags: Flags::ENABLED,
            bundle_required: true,
        };
        let tip_height = zebra_client.get_best_block_height().await?;
        let tree_state = zebra_client.get_orchard_tree_state(tip_height).await?;
        let anchor = Anchor::from_bytes(tree_state.anchor).unwrap();

        let mut builder = OrchardBuilder::new(bundle_type, anchor);

           
        let spendable_note = &spendable_notes[0];
        
        println!("Adding spend action for {} zatoshis", spendable_note.orchard_note.value);

        let fvk = FullViewingKey::from(&spendable_note.spending_key);
        
        
        let note_commitment = spendable_note.orchard_note.note.commitment();
        let note_cmx: orchard::note::ExtractedNoteCommitment = note_commitment.into();
        
        let note_commitment_bytes: [u8; 32] = note_cmx.to_bytes();
        
        let position = zebra_client.get_note_position(&note_commitment_bytes).await?;
        let auth_path = zebra_client.get_authentication_path(position, &tree_state.anchor).await?;
        let mut merkle_hashes: [MerkleHashOrchard; 32] = [MerkleHashOrchard::from_cmx(&note_cmx); 32];

        for (i, hash_bytes) in auth_path.iter().enumerate() {
            if i < 32 {
                
                if let Some(hash) = MerkleHashOrchard::from_bytes(hash_bytes).into() {
                    merkle_hashes[i] = hash;
                } else {
                    return Err(NozyError::MerklePath(format!("Invalid merkle hash at position {}", i)));
                }
            }
        }
        
        let merkle_path = MerklePath::from_parts(position, merkle_hashes);
        builder
            .add_spend(
                fvk,
                spendable_note.orchard_note.note.clone(),
                merkle_path,
            )
            .map_err(|e| NozyError::InvalidOperation(format!("Failed to add spend action: {}", e)))?;
        

        
        let (_, recipient) = zcash_address::unified::Address::decode(recipient_address)
            .map_err(|e| NozyError::AddressParsing(format!("Invalid recipient address: {}", e)))?;

       
        let recipient_orchard_address = {
            let mut orchard_receiver = None;
            for item in recipient.items() {
                if let zcash_address::unified::Receiver::Orchard(data) = item {
                    orchard_receiver = Some(data);
                    break;
                }
            }
            
            match orchard_receiver {
                Some(data) => {
                    OrchardAddress::from_raw_address_bytes(&data)
                        .into_option()
                        .ok_or_else(|| NozyError::AddressParsing("Invalid Orchard receiver in unified address".to_string()))?
                },
                None => return Err(NozyError::AddressParsing("No Orchard receiver found in unified address".to_string())),
            }
        };
        let recipient_note_value = NoteValue::from_raw(amount_zatoshis);
        let recipient_memo = memo.map(|m| {
            let mut memo_bytes = [0u8; 512];
            let len = m.len().min(512);
            memo_bytes[..len].copy_from_slice(&m[..len]);
            memo_bytes
        }).unwrap_or([0u8; 512]);

        builder
            .add_output(
                None,
                recipient_orchard_address,
                recipient_note_value,
                recipient_memo,
            )
            .map_err(|e| NozyError::InvalidOperation(format!("Failed to add recipient output: {}", e)))?;

        if change_amount > 0 {
           
            let change_orchard_address = spendable_note.orchard_note.address;
            let change_note_value = NoteValue::from_raw(change_amount);
            let change_memo = [0u8; 512];

            builder
                .add_output(
                    None,
                    change_orchard_address,
                    change_note_value,
                    change_memo,
                )
                .map_err(|e| NozyError::InvalidOperation(format!("Failed to add change output: {}", e)))?;
        }

        let mut rng = OsRng;
        let bundle_result = builder
        .build::<i64>(&mut rng);

        let (bundle, metadata) = match bundle_result {
            Ok(Some((bundle, metadata))) => (bundle, metadata),
            Ok(None) => return Err(NozyError::InvalidOperation("Failed to build Orchard bundle - no bundle returned".to_string())),
            Err(e) => return Err(NozyError::InvalidOperation(format!("Failed to build Orchard bundle: {}", e))),
        };

        println!("Orchard bundle built successfully!");

        
        println!("Creating Orchard proofs...");
        
        
        let _spending_keys: Vec<SpendingKey> = spendable_notes.iter()
            .map(|note| note.spending_key.clone())
            .collect();
        
       
        
        println!("⚠️  Bundle authorization framework ready");
        println!("⚠️  Note: Full bundle proving requires Orchard parameters");
        println!("⚠️  To complete: Download proving keys and implement proof generation");
        
       
        let mut serialized_transaction = Vec::new();
        
        serialized_transaction.extend_from_slice(&amount_zatoshis.to_le_bytes());
        serialized_transaction.extend_from_slice(&fee_zatoshis.to_le_bytes());
        serialized_transaction.extend_from_slice(&change_amount.to_le_bytes());
        
        serialized_transaction.push(spendable_notes.len() as u8);
        serialized_transaction.push(if change_amount > 0 { 2 } else { 1 }); // outputs count
        
        println!("✅ Orchard transaction structure prepared");
        println!("💡 Transaction size: {} bytes (metadata only)", serialized_transaction.len());
        println!("💡 To broadcast: Implement bundle.create_proof() with Orchard parameters");

        Ok(serialized_transaction)
    }
}