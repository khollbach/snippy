use std::cmp::min;

use rayon::prelude::*;

use crate::{compress::hash_table::HashTable, varint};

/*
TODO:
- ideas for making this faster
    - choose a good size for the hash-table allocation
        (see the reference implementation for heuristics about how big to make it,
         depending on the size of the current block)
    - for small tables, store them on the stack instead
        - todo: get a better understanding of why this is faster
- maybe aim for byte-for-byte compatibility with the reference impl?
    - besides the hash-table stuff, I think we're already reasonably close
    - they're doing some stuff with leaving a >=15-byte literal at the end of
      every block, and maybe other stuff I didn't notice too
        - (somewhat curious to know _why_ this is supposed to be faster)
- we're fairly close in terms of perf now -- our runtime is ~120% of theirs
    - could be cool to run the benchmarks from their readme, and see if there's
      any inputs we do particularly bad on
*/

mod hash_table;

const BLOCK_SIZE: usize = 64 * 1024;
const MAX_COPY_LEN: usize = 64;

/// Panics if input.len() > u32::MAX.
pub fn compress(input: &[u8]) -> Vec<u8> {
    let n = input.len();
    assert!(n <= usize::try_from(u32::MAX).unwrap());
    let n = u32::try_from(n).unwrap();

    // let mut out = Vec::with_capacity(out_buf_capacity(n));

    // Header: uncompressed length.
    let mut result = vec![];

    let mut varint_contents = vec![];
    varint::write(n, &mut varint_contents);
    result.push(varint_contents);

    let output_chunks: Vec<Vec<u8>> = input
        .par_chunks(BLOCK_SIZE)
        .map(|chunk| {
            let mut buffer = Vec::with_capacity(BLOCK_SIZE);

            compress_block(chunk, &mut buffer);

            buffer
        })
        .collect();

    result.into_iter().flatten().collect()
}

/// How much space to pre-allocate in the output buffer.
fn out_buf_capacity(uncompressed_len: u32) -> usize {
    max_compressed_len(uncompressed_len)
        .try_into()
        .unwrap_or(isize::MAX)
        .try_into()
        .unwrap()
}

fn max_compressed_len(uncompressed_len: u32) -> u64 {
    let n = u64::from(uncompressed_len);

    // In the worst case, we have:
    // * 5-byte varint header
    // * the data itself
    // * wasting at most 1 byte every 64 bytes
    let upper_bound = 5 + n + n / 64;

    // The number of wasted bytes is based on the following pattern:
    // * literal of length 61
    // * copy of 4 bytes, at offset >= 2048
    // * ...repeat...
    // which stores 65 bytes of input data as 61 bytes of data plus 5 bytes of
    // metadata:
    // * 2-byte literal tag
    // * 3-byte copy tag
    //
    // Since literals of length < 61 have a 1-byte tag, I don't think you can do
    // better (worse).
    //
    // It also doesn't help to use 3-byte literal tags, since then the literal
    // length needs to be >= 257. And 5-byte copy tags are never used.

    upper_bound
}

fn compress_block(input: &[u8], out: &mut Vec<u8>) {
    let n = input.len();

    let mut seen = HashTable::new(n);
    let mut emitted = 0; // num input bytes compressed so far

    let mut i = 0;
    let i_limit = n.saturating_sub(3); // exclusive
    while i < i_limit {
        let mut next_i = i + 1;

        // Have we seen this 4-byte pattern before?
        let curr_hash = seen.hash(&input[i..i + 4]);
        let i0 = seen.get(curr_hash);
        if i != 0 && input[i..i + 4] == input[i0..i0 + 4] {
            // It's a match!
            let offset = i - i0;
            let len = match_len(input, i, i0);

            if emitted < i {
                emit_literal(&input[emitted..i], out);
                emitted = i;
            }

            emit_copy(offset, len, out);
            emitted += len;

            next_i = min(i + len, i_limit);
        }

        seen.insert(curr_hash, i);
        i += 1;
        while i < next_i {
            let hash = seen.hash(&input[i..i + 4]);
            seen.insert(hash, i);
            i += 1;
        }
    }

    if emitted < n {
        emit_literal(&input[emitted..n], out);
    }
}

/// Extend the match as much as possible: find the first place where the slices
/// differ. Note that the slices might overlap, and that's ok.
fn match_len(input: &[u8], i: usize, i0: usize) -> usize {
    debug_assert!(i0 < i);
    debug_assert_eq!(input[i..i + 4], input[i0..i0 + 4]);

    let mut match_len = 4;
    while match_len < MAX_COPY_LEN
        && i + match_len < input.len()
        && input[i + match_len] == input[i0 + match_len]
    {
        match_len += 1;
    }
    match_len
}

fn emit_literal(data: &[u8], out: &mut Vec<u8>) {
    emit_literal_tag(data.len(), out);
    out.extend_from_slice(data);
}

fn emit_literal_tag(len: usize, out: &mut Vec<u8>) {
    debug_assert!(len != 0);
    debug_assert!(len <= usize::try_from(u32::MAX).unwrap());

    if len <= 60 {
        let len = u8::try_from(len).unwrap();
        let tag_byte = (len - 1) << 2;
        out.push(tag_byte);
    } else {
        let len = u32::try_from(len).unwrap();
        let len_minus_one = len - 1;

        let num_empty_bytes: u32 = len_minus_one.leading_zeros() / 8;
        let num_bytes = 4 - u8::try_from(num_empty_bytes).unwrap();

        let tag_byte = (59 + num_bytes) << 2;
        out.push(tag_byte);

        let buf = len_minus_one.to_le_bytes();
        out.extend_from_slice(&buf[..num_bytes.into()]);
    }
}

fn emit_copy(offset: usize, len: usize, out: &mut Vec<u8>) {
    debug_assert!(offset != 0);
    debug_assert!(offset <= usize::try_from(u32::MAX).unwrap());
    debug_assert!(len != 0);
    debug_assert!(len <= MAX_COPY_LEN);

    // "1-byte" offset
    if (4..=11).contains(&len) && offset < (1 << 11) {
        let len_minus_4 = u8::try_from(len - 4).unwrap();
        assert!(len_minus_4 <= 0x7);

        let offset_high_bits = u8::try_from(offset >> 8).unwrap();
        assert!(offset_high_bits <= 0x7);
        let offset_low_byte = u8::try_from(offset & 0xff).unwrap();

        let tag_byte = offset_high_bits << 5 | len_minus_4 << 2 | 0x1;
        out.push(tag_byte);
        out.push(offset_low_byte);
    }
    // 2-byte offset
    else if let Ok(offset) = u16::try_from(offset) {
        let tag_byte = u8::try_from(len - 1).unwrap() << 2 | 0x2;
        out.push(tag_byte);
        out.extend_from_slice(&offset.to_le_bytes());
    }
    // 4-byte offset
    else {
        let offset = u32::try_from(offset).unwrap();
        let tag_byte = u8::try_from(len - 1).unwrap() << 2 | 0x3;
        out.push(tag_byte);
        out.extend_from_slice(&offset.to_le_bytes());
    }
}
