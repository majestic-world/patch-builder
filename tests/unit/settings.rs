use super::*;

#[test]
fn old_settings_default_to_blake3() {
    let settings: Settings = serde_json::from_str(r#"{"source":"client","update_tree":"updates"}"#).unwrap();
    assert_eq!(settings.source, "client");
    assert_eq!(settings.update_tree, "updates");
    assert_eq!(settings.hash_algorithm, HashAlgorithm::Blake3);
}

#[test]
fn hash_selection_round_trips_with_folders() {
    for (algorithm, serialized) in [
        (HashAlgorithm::Blake3, "blake3"),
        (HashAlgorithm::Crc32, "crc32"),
        (HashAlgorithm::Both, "blake3+crc32"),
    ] {
        let settings = Settings {
            source: "client".into(),
            update_tree: "updates".into(),
            hash_algorithm: algorithm,
        };
        let json = serde_json::to_value(&settings).unwrap();
        assert_eq!(json["hash_algorithm"], serialized);
        assert_eq!(serde_json::from_value::<Settings>(json).unwrap(), settings);
    }
}
