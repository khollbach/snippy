use std::{cmp::min, io::Read};

use anyhow::{Context, Result, ensure};

use crate::{
    decompress::tag::{CopyTag, LiteralTag, Tag},
    varint,
};

mod tag;

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
        let tag_byte = buf[0];

        match tag::parse(tag_byte) {
            Tag::Literal(tag) => literal(&mut r, tag, &mut out)?,
            Tag::Copy(tag) => copy(&mut r, tag, &mut out)?,
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

fn literal<R: Read>(mut r: R, tag: LiteralTag, out: &mut Vec<u8>) -> Result<()> {
    let len: u32 = read_literal_len(&mut r, tag).context("read literal len")?;
    let len = usize::try_from(len).unwrap();

    let curr_offset = out.len();
    let new_len = curr_offset + len;
    let total_len = out.capacity();
    ensure!(
        new_len <= total_len,
        "curr len plus literal len overflows total len: {} + {} = {} > {}",
        curr_offset,
        len,
        new_len,
        total_len,
    );

    out.resize(new_len, 0);
    r.read_exact(&mut out[curr_offset..])
        .context("EOF while reading literal")?;

    Ok(())
}

fn read_literal_len<R: Read>(mut r: R, tag: LiteralTag) -> Result<u32> {
    match tag {
        LiteralTag::LengthValue(len) => return Ok(len.into()),
        LiteralTag::LengthNumBytes(width) => {
            let mut buf = [0; 4];
            r.read_exact(&mut buf[..width.into()])
                .context("unexpected EOF")?;

            let len = u32::from_le_bytes(buf)
                .checked_add(1)
                .context("literal len must not equal 2^32 (since total len must not equal 2^32)")?;
            Ok(len)
        }
    }
}

fn copy<R: Read>(mut r: R, tag: CopyTag, out: &mut Vec<u8>) -> Result<()> {
    let offset: u32 = read_copy_offset(&mut r, tag).context("read copy offset")?;
    let offset = usize::try_from(offset).unwrap();
    let len = usize::from(tag.len);

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
    let total_len = out.capacity();
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

    Ok(())
}

fn read_copy_offset<R: Read>(mut r: R, tag: CopyTag) -> Result<u32> {
    let mut buf = [0; 4];
    r.read_exact(&mut buf[..tag.offset_num_bytes.into()])
        .context("unexpected EOF")?;
    let offset = u32::from_le_bytes(buf);
    let high_bits = u32::from(tag.offset_high_bits);
    Ok(offset | high_bits)
}
