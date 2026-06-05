use std::{cmp::min, io::Read};

use anyhow::{Context, Result, ensure};

use crate::{decompress::tag::Tag, varint};

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

pub fn decompress<R: Read>(r: &mut R) -> Result<Vec<u8>> {
    let total_len: u32 = varint::read(r).context("read total len")?;
    let total_len = usize::try_from(total_len).unwrap();

    let mut out = Vec::with_capacity(total_len);

    while out.len() < total_len {
        let tag = tag::read(r)
            .with_context(|| format!("decoded {} of {} bytes", out.len(), total_len))?;

        match tag {
            Tag::Literal { len } => decompress_literal(r, len, &mut out)?,
            Tag::Copy { offset, len } => decompress_copy(offset, len, &mut out)?,
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

fn decompress_literal<R: Read>(r: &mut R, len: u32, out: &mut Vec<u8>) -> Result<()> {
    let len = usize::try_from(len).unwrap();

    ensure!(
        out.len() + len <= out.capacity(),
        "curr len plus literal len overflows total len: {} + {} = {} > {}",
        out.len(),
        len,
        out.len() + len,
        out.capacity(),
    );

    let start = out.len();

    // out.resize(out.len() + len, 0);
    unsafe {
        out.set_len(out.len() + len);
    };

    r.read_exact(&mut out[start..])
        .context("EOF while reading literal")?;

    Ok(())
}

fn decompress_copy(offset: u32, len: u8, out: &mut Vec<u8>) -> Result<()> {
    let offset = usize::try_from(offset).unwrap();
    let len = usize::from(len);

    ensure!(offset != 0, "copy offset of 0 is invalid");
    ensure!(
        offset <= out.len(),
        "offset past beginning of input: {} vs {}",
        offset,
        out.len()
    );
    ensure!(
        out.len() + len <= out.capacity(),
        "curr len plus copy len overflows total len: {} + {} = {} > {}",
        out.len(),
        len,
        out.len() + len,
        out.capacity(),
    );

    let slice_start = out.len() - offset;
    let slice_len = min(len, offset);
    let finished_len = out.len() + len;

    // Append `out[slice_start..][..slice_len]`, possibly many times.
    while out.len() < finished_len {
        let copy_len = min(slice_len, finished_len - out.len());
        out.extend_from_within(slice_start..slice_start + copy_len);
    }
    debug_assert_eq!(out.len(), finished_len);

    Ok(())
}
