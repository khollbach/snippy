use std::io::Read;

use anyhow::{Context, Result, ensure};

pub fn read<R: Read>(mut r: R) -> Result<u32> {
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

pub fn write(mut x: u32, out: &mut Vec<u8>) {
    loop {
        let low_bits = x & 0x7f;
        let high_bit = if x == low_bits { 0 } else { 0x80 };
        out.push(high_bit | u8::try_from(low_bits).unwrap());

        x >>= 7;
        if x == 0 {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(&[0], 0; "zero")]
    #[test_case(&[0x40], 64; "64")]
    #[test_case(&[0xfe, 0xff, 0x7f], 0x1f_fffe; "0x1f_fffe")]
    #[test_case(&[0xff, 0xff, 0xff, 0xff, 0xf], 0xffff_ffff; "u32 max")]
    fn read_ok(mut buf: &[u8], expected: u32) -> Result<()> {
        assert_eq!(read(&mut buf)?, expected);
        assert_eq!(buf, []);
        Ok(())
    }

    #[test_case(&[0x80, 0x80, 0x80, 0x80, 0x10]; "overflow")]
    #[test_case(&[0x80, 0x80, 0x80, 0x80, 0x80]; "too many bytes")]
    fn read_err(buf: &[u8]) {
        read(buf).unwrap_err();
    }

    #[test_case(&[0], 0; "zero")]
    #[test_case(&[0x40], 64; "64")]
    #[test_case(&[0xfe, 0xff, 0x7f], 0x1f_fffe; "0x1f_fffe")]
    #[test_case(&[0xff, 0xff, 0xff, 0xff, 0xf], 0xffff_ffff; "u32 max")]
    fn write(expected: &[u8], x: u32) {
        let mut buf = vec![];
        super::write(x, &mut buf);
        assert_eq!(buf, expected);
    }
}
