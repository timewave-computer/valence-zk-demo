use sha2::{Digest, Sha256};
pub(crate) fn merkleize_container(field_roots: Vec<[u8; 32]>) -> [u8; 32] {
    let count = field_roots.len();
    let next_pow2 = count.next_power_of_two();
    let mut leaves = field_roots;
    leaves.extend(vec![[0u8; 32]; next_pow2 - count]);
    while leaves.len() > 1 {
        let mut next_level = vec![];
        for i in (0..leaves.len()).step_by(2) {
            let mut hasher = Sha256::new();
            hasher.update(leaves[i]);
            hasher.update(leaves[i + 1]);
            next_level.push(hasher.finalize().into());
        }
        leaves = next_level;
    }
    leaves[0]
}
