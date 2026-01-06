// Safe display utilities for sensitive data
// Prevents accidental exposure of private keys, mnemonics, passwords, etc.

pub fn display_mnemonic_safe(mnemonic: &str) -> String {
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    let word_count = words.len();
    
    if word_count == 0 {
        return "[empty]".to_string();
    }
    
    if word_count <= 6 {
        // For short mnemonics, show first 2 and last 1
        if word_count <= 3 {
            format!("{}...", words[0])
        } else {
            format!("{} {} ... {}", words[0], words[1], words[word_count - 1])
        }
    } else {
        // For standard mnemonics (12/24 words), show first 4 and last 1
        format!(
            "{} {} {} {} ... {}",
            words[0],
            words[1],
            words[2],
            words[3],
            words[word_count - 1]
        )
    }
}

/// This should only be called in secure contexts where user explicitly requested it
pub fn display_mnemonic_full(mnemonic: &str) -> String {
    mnemonic.to_string()
}

pub fn mask_string(s: &str, visible_start: usize, visible_end: usize) -> String {
    if s.len() <= visible_start + visible_end {
        return "*".repeat(s.len());
    }
    
    format!(
        "{}...{}",
        &s[..visible_start],
        &s[s.len() - visible_end..]
    )
}

pub fn mask_private_key(key: &[u8]) -> String {
    if key.len() <= 8 {
        return "****".to_string();
    }
    
    let hex = hex::encode(key);
    mask_string(&hex, 4, 4)
}

pub fn mask_address(address: &str) -> String {
    mask_string(address, 8, 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_mnemonic_safe() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let safe = display_mnemonic_safe(mnemonic);
        assert!(safe.contains("abandon"));
        assert!(safe.contains("about"));
        assert!(!safe.contains("abandon abandon abandon abandon abandon abandon abandon abandon"));
    }

    #[test]
    fn test_mask_string() {
        let s = "abcdefghijklmnopqrstuvwxyz";
        let masked = mask_string(s, 3, 3);
        assert_eq!(masked, "abc...xyz");
    }

    #[test]
    fn test_mask_private_key() {
        let key = b"0123456789abcdef0123456789abcdef";
        let masked = mask_private_key(key);
        assert!(masked.contains("..."));
        assert!(masked.len() < hex::encode(key).len());
    }
}

/*Note in case we have problems in future 