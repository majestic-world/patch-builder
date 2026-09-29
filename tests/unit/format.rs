use super::*;

#[test]
fn count_groups_thousands() {
    assert_eq!(count(0), "0");
    assert_eq!(count(999), "999");
    assert_eq!(count(1_000), "1,000");
    assert_eq!(count(20_034), "20,034");
    assert_eq!(count(1_234_567), "1,234,567");
}

#[test]
fn bytes_below_one_kilobyte_stay_exact() {
    assert_eq!(bytes(0), "0 B");
    assert_eq!(bytes(1_023), "1023 B");
}

#[test]
fn bytes_use_binary_units_with_one_decimal() {
    assert_eq!(bytes(1_024), "1.0 KB");
    assert_eq!(bytes(29_779_558), "28.4 MB");
    assert_eq!(bytes(12_455_405_158), "11.6 GB");
}

#[test]
fn bytes_promote_instead_of_rounding_to_1024() {
    // 1023.96 KB would print as "1024.0 KB".
    assert_eq!(bytes(1_048_535), "1.0 MB");
    assert_eq!(bytes(1_048_524), "1023.9 KB");
}

#[test]
fn bytes_cap_at_the_largest_unit() {
    assert_eq!(bytes(u64::MAX), "16777216.0 TB");
}

#[test]
fn noun_is_singular_only_for_one() {
    assert_eq!(noun(0, "file", "files"), "files");
    assert_eq!(noun(1, "file", "files"), "file");
    assert_eq!(noun(2, "file", "files"), "files");
}

#[test]
fn short_hash_keeps_short_input_whole() {
    assert_eq!(short_hash("a3f1e7c9b2d4e6f10b8c7d5e4f3a2b1c"), "a3f1e7c9b2d4e6f1");
    assert_eq!(short_hash("abc"), "abc");
}
