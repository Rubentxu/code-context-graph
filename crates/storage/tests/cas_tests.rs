/// TDD contract for CAS store
/// Expected API (to be implemented):
/// - code_context_graph_storage::cas::{CasStore, CasConfig}
/// - CasStore::new(config) -> anyhow::Result<Self>
/// - put_bytes(&self, data: &[u8]) -> anyhow::Result<String>  // returns content hash
/// - get(&self, hash: &str) -> anyhow::Result<Option<Vec<u8>>>
/// - has(&self, hash: &str) -> anyhow::Result<bool>
/// - deduplication: storing same bytes twice yields same hash and single copy on disk
#[test]
fn cas_put_get_and_deduplication() {
    use code_context_graph_storage::cas::{CasConfig, CasStore};

    let temp = tempfile::tempdir().unwrap();
    let cas_path = temp.path().join("cas");

    // It should not exist before initialization
    assert!(!cas_path.exists());

    let cfg = CasConfig { root: cas_path.clone() };
    let cas = CasStore::new(cfg).expect("CAS should initialize");

    let h1 = cas.put_bytes(b"hello world").unwrap();
    let h2 = cas.put_bytes(b"hello world").unwrap();
    assert_eq!(h1, h2, "same content must produce same hash");

    assert!(cas.has(&h1).unwrap());
    let roundtrip = cas.get(&h1).unwrap().expect("content must exist");
    assert_eq!(roundtrip, b"hello world");
}
