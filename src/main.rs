use anyhow::Result;

fn main() -> Result<()> {
    let input = std::fs::read("input.txt").unwrap();
    let compressed = snippy::compress(&input);
    std::fs::write("output.txt", &compressed).unwrap();
    let decompressed = snippy::decompress(&compressed)?;
    assert_eq!(input, decompressed);
    Ok(())
}
