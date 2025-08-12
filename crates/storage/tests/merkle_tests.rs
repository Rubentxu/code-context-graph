/// TDD contract for Merkle tree versioning (minimal)
/// Expected API (to be implemented):
/// - code_context_graph_storage::merkle::{MerkleBuilder, MerkleTree, Diff}
#[test]
fn merkle_builds_root_and_diff() {
    use code_context_graph_storage::merkle::MerkleBuilder;

    // v1
    let mut b1 = MerkleBuilder::new().fanout(16);
    b1.add("a.txt", b"aaa");
    b1.add("b.txt", b"bbb");
    let t1 = b1.build();
    let r1 = t1.root();

    // v2 modify one file
    let mut b2 = MerkleBuilder::new().fanout(16);
    b2.add("a.txt", b"xxx");
    b2.add("b.txt", b"bbb");
    let t2 = b2.build();
    let r2 = t2.root();

    assert_ne!(r1, r2, "root must change when content changes");

    let diff = t1.diff(&t2);
    assert_eq!(diff.changed_paths, vec!["a.txt".to_string()]);
}
