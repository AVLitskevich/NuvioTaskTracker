//! Telegram WebApp initData verification.
//!
//! Algorithm (official spec):
//!   1. Parse initData as URL query string.
//!   2. Extract `hash` field; remove it from the set.
//!   3. Sort remaining key=value pairs lexicographically.
//!   4. Join with '\n' → data_check_string.
//!   5. secret_key = HMAC-SHA256(key = b"WebAppData", data = bot_token_bytes)
//!   6. expected   = HMAC-SHA256(key = secret_key,   data = data_check_string)
//!   7. Constant-time compare hex(expected) == hash.
//!   8. Reject if auth_date > MAX_AGE_SECS seconds old.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::BTreeMap;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// Maximum age of a valid initData payload (Telegram recommends ≤ 86 400 s).
/// Set to 300 seconds (5 minutes) to prevent replay attacks.
const MAX_AGE_SECS: u64 = 300;

#[derive(Debug, Error)]
pub enum InitDataError {
    #[error("missing hash field")]
    MissingHash,
    #[error("missing auth_date field")]
    MissingAuthDate,
    #[error("auth_date is not a valid integer")]
    InvalidAuthDate,
    #[error("initData expired (age {age}s > max {MAX_AGE_SECS}s)")]
    Expired { age: u64 },
    #[error("signature mismatch")]
    SignatureMismatch,
}

/// Validate Telegram WebApp initData.
///
/// Returns the parsed fields (excluding `hash`) on success.
pub fn validate_init_data(
    init_data: &str,
    bot_token: &str,
    now_unix: u64,
) -> Result<BTreeMap<String, String>, InitDataError> {
    // 1. Parse query string into sorted map
    let mut fields: BTreeMap<String, String> = form_urlencoded::parse(init_data.as_bytes())
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    // 2. Extract hash
    let hash = fields.remove("hash").ok_or(InitDataError::MissingHash)?;

    // 3. Check auth_date freshness
    let auth_date: u64 = fields
        .get("auth_date")
        .ok_or(InitDataError::MissingAuthDate)?
        .parse()
        .map_err(|_| InitDataError::InvalidAuthDate)?;

    let age = now_unix.saturating_sub(auth_date);
    if age > MAX_AGE_SECS {
        return Err(InitDataError::Expired { age });
    }

    // 4. Build data_check_string (BTreeMap is already sorted)
    let data_check_string = fields
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");

    // 5. secret_key = HMAC-SHA256(key="WebAppData", data=bot_token)
    let mut mac = HmacSha256::new_from_slice(b"WebAppData")
        .expect("HMAC accepts any key length");
    mac.update(bot_token.as_bytes());
    let secret_key = mac.finalize().into_bytes();

    // 6. expected = HMAC-SHA256(key=secret_key, data=data_check_string)
    let mut mac2 = HmacSha256::new_from_slice(&secret_key)
        .expect("HMAC accepts any key length");
    mac2.update(data_check_string.as_bytes());
    let expected = mac2.finalize().into_bytes();

    // 7. Constant-time comparison via hex strings
    let expected_hex = hex::encode(expected);
    if !constant_time_eq(expected_hex.as_bytes(), hash.as_bytes()) {
        return Err(InitDataError::SignatureMismatch);
    }

    Ok(fields)
}

/// Constant-time byte comparison (prevents timing attacks).
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}
