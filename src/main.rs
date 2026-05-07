use anyhow::Result;

fn main() -> Result<()> {
    let input = b"hello world".as_slice();
    let compressed = snippy::compress(input);
    let decompressed = snippy::decompress(&compressed)?;
    assert_eq!(input, decompressed);
    Ok(())
}
