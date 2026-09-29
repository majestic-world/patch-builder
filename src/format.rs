//! Human-readable formatting for counts, sizes and hashes shown in the UI.

const SIZE_UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

/// Hex digits shown for a hash in tables.
const SHORT_HASH_LEN: usize = 16;

/// Formats `n` with `,` thousands separators (`20034` → `20,034`).
pub fn count(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// Formats a byte count with binary multiples and one decimal (`29779558` → `28.4 MB`).
pub fn bytes(n: u64) -> String {
    if n < 1024 {
        return format!("{n} B");
    }
    let mut value = n as f64;
    let mut unit = 0;
    // Promote while the value would round to 1024.0 in the current unit.
    while value >= 1023.95 && unit < SIZE_UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", SIZE_UNITS[unit])
}

/// Picks the noun form matching `n`.
pub fn noun<'a>(n: u64, singular: &'a str, plural: &'a str) -> &'a str {
    if n == 1 { singular } else { plural }
}

/// Leading hex digits of a hash, enough to tell files apart at a glance.
pub fn short_hash(hash: &str) -> &str {
    hash.get(..SHORT_HASH_LEN).unwrap_or(hash)
}

#[cfg(test)]
#[path = "../tests/unit/format.rs"]
mod tests;
