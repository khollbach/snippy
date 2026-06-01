use std::{io::Read, ptr};

use anyhow::{Context, Result, ensure};

use crate::{
    decompress::tag::{CopyTag, LiteralTag, Tag},
    varint,
};

/*
TODO: perf ideas
- in literal-copy code, use unsafe to avoid extra zeroing of output buffer
- idea from Ty: set up a guard page, and then just write without checking any bounds ever
    - need to also handle the signal that the OS sends us when we hit it
    - overall this sounds *very* involved, but could be really cool to look into
- from rust-snappy source code:
    - read 4-byte integer always, and then mask away the high bytes depending on desired width
    - SIMD stuff...
        - writing loops in weird ways to encourage compiler to vectorize them
        - & copying more than you need to sometimes, knowing that it's OK, so that
          it gets vectorized
    - being thoughtful about when & where bounds checks are happening / necessary
    - pre-computing tag-byte info at compile time & looking it up at runtime
        - I tried this, but it actually slowed us down -- idk what I did wrong...
    - (maybe more things? see the code for ideas)
- could also run their benchmarks to see which inputs are especially bad, & maybe
  that would give us some ideas
*/

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

        match Tag::parse(tag_byte) {
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
    let start = out.len() - offset;

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

    unsafe {
        let src = out.as_ptr().offset(isize::try_from(start).unwrap());
        let dst = out.as_mut_ptr().offset(isize::try_from(out.len()).unwrap());
        ptr::copy(src, dst, len);

        out.set_len(finished_len);
    }

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
