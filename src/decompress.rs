use std::{cmp::min, io::Read};

use anyhow::{Context, Result, ensure};

use crate::varint;

pub fn decompress<R: Read>(mut r: R) -> Result<Vec<u8>> {
    let total_len: u32 = varint::read(&mut r).context("read total len")?;
    let total_len = usize::try_from(total_len).unwrap();
    let mut out = Vec::with_capacity(total_len);

    while out.len() < total_len {
        let mut buf = [0];
        r.read_exact(&mut buf).with_context(|| {
            format!(
                "unexpected EOF; decoded {} of {} bytes",
                out.len(),
                total_len,
            )
        })?;
        let tag = buf[0];

        if tag & 0x3 == 0x0 {
            let lit_len: u32 = read_literal_len(&mut r, tag).context("read literal len")?;

            let curr_offset = out.len();
            let new_len = curr_offset + usize::try_from(lit_len).unwrap();
            ensure!(
                new_len <= total_len,
                "curr len plus literal len overflows total len: {} + {} = {} > {}",
                curr_offset,
                lit_len,
                new_len,
                total_len,
            );

            // // todo -- is there a faster way? we don't really *need* to write zeros...
            // out.resize(new_len, 0);
            // r.read_exact(&mut out[curr_offset..])
            //     .context("EOF while reading literal")?;

            (&mut r).take(u64::from(lit_len)).read_to_end(&mut out)?;
        } else {
            let (offset, len): (u32, _) = read_copy_tag(&mut r, tag).context("read copy tag")?;
            let offset = usize::try_from(offset).unwrap();
            let len = usize::from(len);

            ensure!(offset != 0, "copy offset of 0 is invalid");
            ensure!(
                offset <= out.len(),
                "offset past beginning of input: {} vs {}",
                offset,
                out.len()
            );
            let slice_start = out.len() - offset;
            let slice_len = min(offset, len);

            let finished_len = out.len() + len;
            ensure!(
                finished_len <= total_len,
                "curr len plus copy len overflows total len: {} + {} = {} > {}",
                out.len(),
                len,
                finished_len,
                total_len,
            );

            // Append `out[slice_start..][..slice_len]`, possibly many times.
            while out.len() < finished_len {
                let copy_len = min(slice_len, finished_len - out.len());
                out.extend_from_within(slice_start..slice_start + copy_len);
            }
            debug_assert_eq!(out.len(), finished_len);
        }
    }
    ensure!(
        out.len() == total_len,
        "decompressed output longer than expected: {} vs {}",
        out.len(),
        total_len
    );

    Ok(out)
}

fn read_literal_len<R: Read>(mut r: R, tag: u8) -> Result<u32> {
    debug_assert_eq!(tag & 0x3, 0x0);
    let width = match tag >> 2 {
        60 => 1,
        61 => 2,
        62 => 3,
        63 => 4,
        n => return Ok(u32::from(n) + 1),
    };

    let mut buf = [0; 4];
    r.read_exact(&mut buf[..width]).context("unexpected EOF")?;

    let len = u32::from_le_bytes(buf)
        .checked_add(1)
        .context("literal len must not equal 2^32 (since total len must not equal 2^32)")?;
    Ok(len)
}

/// (offset, len)
fn read_copy_tag<R: Read>(mut r: R, tag: u8) -> Result<(u32, u8)> {
    let offset_width = match tag & 0x3 {
        0 => panic!("read_copy_tag called with literal tag byte"),
        1 => {
            let len = (tag >> 2 & 0x7) + 4; // middle 3 bits
            debug_assert!((4..=11).contains(&len));

            let mut buf = [0];
            r.read_exact(&mut buf).context("unexpected EOF")?;
            let offset_hi = tag >> 5; // high 3 bits
            let offset_lo = buf[0];

            let offset = u16::from_le_bytes([offset_lo, offset_hi]);
            return Ok((u32::from(offset), len));
        }
        2 => 2,
        3 => 4,
        _ => unreachable!(),
    };

    let len = (tag >> 2) + 1;

    let mut buf = [0; 4];
    r.read_exact(&mut buf[..offset_width])
        .context("unexpected EOF")?;
    let offset = u32::from_le_bytes(buf);

    Ok((offset, len))
}
