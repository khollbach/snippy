use anyhow::Result;

fn main() -> Result<()> {
    let input = b"hello world".as_slice();
    dbg!(input);
    let compressed = rippy::compress(input);
    dbg!(&compressed);
    let decompressed = rippy::decompress(&compressed)?;
    dbg!(&decompressed);
    assert_eq!(input, decompressed);
    Ok(())
}
