use trapiks_sim_wasm::{ChallengeRunner, Engine, MapHandle};

#[test]
fn challenge_bindings_compile() {
    let sizes = [
        std::mem::size_of::<MapHandle>(),
        std::mem::size_of::<ChallengeRunner>(),
        std::mem::size_of::<Engine>(),
    ];
    assert!(sizes.iter().all(|&size| size > 0));
}
