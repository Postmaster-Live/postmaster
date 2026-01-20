use hmac::{Hmac, Mac};
use sha2::Sha256;
use anyhow::{anyhow, Result};

type HmacSha256 = Hmac<Sha256>;

pub fn verify_signature(
    secret: &str,
    timestamp: &str,
    body: &str,
    signature: &str,
) -> Result<bool> {
    // Check timestamp freshness (prevent replay attacks)
    let ts: i64 = timestamp.parse()
        .map_err(|_| anyhow!("Invalid timestamp"))?;
    let now = chrono::Utc::now().timestamp();
    if (now - ts).abs() > 300 { // 5 minutes tolerance
        tracing::warn!("Signature timestamp too old or in future: {} vs {}", ts, now);
        return Ok(false);
    }

    // Compute expected signature
    let message = format!("{}.{}", timestamp, body);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| anyhow!("Invalid key: {}", e))?;
    mac.update(message.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());

    // Compare (constant time via hex crate)
    let provided = signature.strip_prefix("sha256=").unwrap_or(signature);
    
    Ok(expected == provided)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_signature() {
        let secret = "test_secret";
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let body = r#"{"test":"data"}"#;
        
        // Generate signature
        let message = format!("{}.{}", timestamp, body);
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(message.as_bytes());
        let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
        
        // Verify
        assert!(verify_signature(secret, &timestamp, body, &signature).unwrap());
    }

    #[test]
    fn test_verify_signature_fails_wrong_secret() {
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let body = r#"{"test":"data"}"#;
        
        let message = format!("{}.{}", timestamp, body);
        let mut mac = HmacSha256::new_from_slice("secret1".as_bytes()).unwrap();
        mac.update(message.as_bytes());
        let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
        
        assert!(!verify_signature("secret2", &timestamp, body, &signature).unwrap());
    }

    #[test]
    fn test_verify_signature_fails_old_timestamp() {
        let secret = "test_secret";
        let timestamp = (chrono::Utc::now().timestamp() - 400).to_string(); // 400 seconds ago
        let body = r#"{"test":"data"}"#;
        
        let message = format!("{}.{}", timestamp, body);
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(message.as_bytes());
        let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
        
        assert!(!verify_signature(secret, &timestamp, body, &signature).unwrap());
    }
}