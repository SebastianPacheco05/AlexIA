use alexia::secrets::{OsSecretStore, SecretStore};

#[test]
fn vault_roundtrip_never_writes_a_plaintext_fallback() {
    let store = OsSecretStore;
    let reference = format!("test-{}", alexia::auth::random_id());
    #[cfg(windows)]
    {
        store.put(&reference, "synthetic-sentinel").unwrap();
        assert_eq!(store.get(&reference).unwrap(), "synthetic-sentinel");
        store.delete(&reference).unwrap();
        assert!(store.get(&reference).is_err());
    }
    #[cfg(not(windows))]
    assert!(store.put(&reference, "synthetic-sentinel").is_err());
}
