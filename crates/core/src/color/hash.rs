//! Deterministic hashing functions for reproducible color assignment.

use uuid::Uuid;

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

/// Computes a deterministic 64-bit FNV-1a hash over the provided byte slice.
///
/// FNV-1a guarantees identical hash values across different platforms, process runs,
/// architectures, and endianness.
///
/// # Examples
///
/// ```
/// use newsjournal_core::color::fnv1a_hash;
///
/// let hash1 = fnv1a_hash(b"city-budget-2026");
/// let hash2 = fnv1a_hash(b"city-budget-2026");
/// assert_eq!(hash1, hash2);
/// ```
#[must_use]
pub fn fnv1a_hash(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Computes a deterministic 64-bit FNV-1a hash for a string slice.
///
/// Strings are normalized by trimming whitespace before hashing.
///
/// # Examples
///
/// ```
/// use newsjournal_core::color::fnv1a_hash_str;
///
/// let hash = fnv1a_hash_str("investigative-reporting");
/// assert_ne!(hash, 0);
/// ```
#[must_use]
pub fn fnv1a_hash_str(s: &str) -> u64 {
    fnv1a_hash(s.trim().as_bytes())
}

/// Computes a deterministic 64-bit FNV-1a hash for a UUID.
///
/// # Examples
///
/// ```
/// use newsjournal_core::color::fnv1a_hash_uuid;
/// use uuid::Uuid;
///
/// let id = Uuid::nil();
/// let hash = fnv1a_hash_uuid(&id);
/// assert_ne!(hash, 0);
/// ```
#[must_use]
pub fn fnv1a_hash_uuid(uuid: &Uuid) -> u64 {
    fnv1a_hash(uuid.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fnv1a_empty_bytes() {
        assert_eq!(fnv1a_hash(b""), FNV_OFFSET_BASIS);
    }

    #[test]
    fn test_fnv1a_deterministic_string_hashes() {
        let h1 = fnv1a_hash_str("transit-investigation");
        let h2 = fnv1a_hash_str("transit-investigation");
        let h3 = fnv1a_hash_str("  transit-investigation  ");
        assert_eq!(h1, h2);
        assert_eq!(h1, h3);

        let h_diff = fnv1a_hash_str("other-story");
        assert_ne!(h1, h_diff);
    }

    #[test]
    fn test_fnv1a_uuid() {
        let u1 = Uuid::new_v4();
        let u2 = u1;
        assert_eq!(fnv1a_hash_uuid(&u1), fnv1a_hash_uuid(&u2));
    }
}
