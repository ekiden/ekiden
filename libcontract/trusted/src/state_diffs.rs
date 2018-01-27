fn diff(old: &[u8], new: &[u8]) -> Vec<u8> {
    new.to_vec()
}

fn apply(old: &[u8], diff: &[u8]) -> Vec<u8> {
    diff.to_vec()
}
