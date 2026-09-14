use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Computes the HMAC-SHA256 digest of `data` using `key`, returning a lowercase hex string.
pub fn hmac_sha256_hex(key: &[u8], data: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC can accept keys of any arbitrary length");
    mac.update(data);
    hex::encode(mac.finalize().into_bytes())
}

/// Derives the static default AES encryption key used by Virat.
///
/// Matches Virat's NestJS implementation:
/// `CryptoJS.HmacSHA256(secret, hashKey).toString(CryptoJS.enc.Hex)`
pub fn derive_default_aes_key(secret: &str, hash_key: &str) -> String {
    hmac_sha256_hex(hash_key.as_bytes(), secret.as_bytes())
}

/// Derives the dynamic per-response AES key used by Virat's outbound logger interceptor.
///
/// In Virat: `CryptoJS.HmacSHA256(secret + ip + timestamp, hashKey).toString(CryptoJS.enc.Hex)`
///
/// Note: By calling `update()` multiple times on the HMAC state, we stream the bytes directly
/// into the hash algorithm without allocating an intermediate concatenated `String` on the heap!
pub fn derive_dynamic_aes_key(secret: &str, ip: &str, timestamp: &str, hash_key: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(hash_key.as_bytes())
        .expect("HMAC can accept keys of any arbitrary length");
    mac.update(secret.as_bytes());
    mac.update(ip.as_bytes());
    mac.update(timestamp.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_default_aes_key_matches_virat() {
        // Known Virat credentials from dev configuration
        let secret = "5b6714126d3149fbab994747b2633287";
        let hash_key = "r4vcos0ejvndsow95n";

        let derived = derive_default_aes_key(secret, hash_key);

        // Verified against NodeJS CryptoJS.HmacSHA256(secret, hashKey).toString(CryptoJS.enc.Hex)
        assert_eq!(
            derived,
            "aaa362430cc8b40f9ca4f3798dc008057cebb722a51e7771eed5a5fdafc4d976"
        );
    }

    #[test]
    fn test_derive_dynamic_aes_key_matches_virat() {
        let secret = "5b6714126d3149fbab994747b2633287";
        let hash_key = "r4vcos0ejvndsow95n";
        let ip = "103.21.244.2";
        let timestamp = "1726056789";

        let derived = derive_dynamic_aes_key(secret, ip, timestamp, hash_key);

        // Verified against NodeJS CryptoJS output
        assert_eq!(
            derived,
            "d0f3313cdfa96d203cc081e11bbe1bf41b7887dbed7145cac717ca5f33074e00"
        );
    }
}
