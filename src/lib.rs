use anyhow::Result;

mod compress;
mod decompress;
mod varint;

pub fn compress(input: &[u8]) -> Vec<u8> {
    compress::compress(input)
}

pub fn decompress(compressed: &[u8]) -> Result<Vec<u8>> {
    decompress::decompress(compressed)
}
