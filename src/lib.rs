use anyhow::Result;

pub fn compress(input: &[u8]) -> Vec<u8> {
    input.to_vec()
}

pub fn decompress(compressed: &[u8]) -> Result<Vec<u8>> {
    Ok(compressed.to_vec())
}
