use std::io::Read;

use anyhow::{Context, Result, ensure};

pub fn decompress(mut compressed: &[u8]) -> Result<Vec<u8>> {
    let len = read_varint(&mut compressed)?;

    todo!()
}

fn read_varint<R: Read>(mut r: R) -> Result<u32> {
    let mut out = 0;

    for i in 0.. {
        let mut buf = [0];
        r.read_exact(&mut buf).context("EOF while reading varint")?;
        let byte = buf[0];
        if i == 4 {
            ensure!(byte & 0xf0 == 0, "varint overflows u32");
        }

        out |= u32::from(byte & 0x7f) << i * 7;
        if byte & 0x80 == 0 {
            break;
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(&[0], 0; "zero")]
    #[test_case(&[0x40], 64; "64")]
    #[test_case(&[0xfe, 0xff, 0x7f], 0x1f_fffe; "0x1f_fffe")]
    #[test_case(&[0xff, 0xff, 0xff, 0xff, 0xf], 0xffff_ffff; "u32 max")]
    fn read_varint(mut buf: &[u8], expected: u32) -> Result<()> {
        assert_eq!(super::read_varint(&mut buf)?, expected);
        assert_eq!(buf, []);
        Ok(())
    }

    #[test_case(&[0x80, 0x80, 0x80, 0x80, 0x10]; "overflow")]
    #[test_case(&[0x80, 0x80, 0x80, 0x80, 0x80]; "too many bytes")]
    fn read_varint_err(buf: &[u8]) {
        super::read_varint(buf).unwrap_err();
    }
}
