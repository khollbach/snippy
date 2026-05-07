use anyhow::Result;

fn main() -> Result<()> {
    let input = b"hello world".as_slice();
    dbg!(input);
    let compressed = snippy::compress(input);
    dbg!(&compressed);
    let decompressed = snippy::decompress(&compressed)?;
    dbg!(&decompressed);
    assert_eq!(input, decompressed);
    Ok(())
}
