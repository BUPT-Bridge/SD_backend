use md5::{Digest, Md5};

/// Encrypt plain text password using MD5 hash
///
/// # Arguments
/// * `password` - The plain text password to encrypt
///
/// # Returns
/// The MD5 hash of the password as a hex string
pub fn md5_hash_password(password: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5_hash_password() {
        let password = "123456";
        let hash = md5_hash_password(password);
        // MD5 of "123456" should be "e10adc3949ba59abbe56e057f20f883e"
        assert_eq!(hash, "e10adc3949ba59abbe56e057f20f883e");
    }

    #[test]
    fn test_same_password_same_hash() {
        let password = "test123";
        let hash1 = md5_hash_password(password);
        let hash2 = md5_hash_password(password);
        assert_eq!(hash1, hash2);
    }
}
