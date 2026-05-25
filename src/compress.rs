use std::cmp::min;

use rayon::{iter::ParallelIterator, slice::ParallelSlice};

use crate::{compress::hash_table::HashTable, varint};

/*
TODO:
- try to make this as fast as the reference implementation?
  ideas (cribbed from the code of rust-snappy):
    - choose a good size for the hash-table allocation
        (see their code for heuristics about how big to make it,
         depending on the size of the current block)
    - for small tables, store them on the stack instead
        - todo: get a better understanding of why this is faster
    - unchecked input-reads didn't seem to make a difference, but
      maybe unchecked output-writes would? Worth a try!
- maybe aim for byte-for-byte compatibility with the reference impl?
    - besides the hash-table stuff, I think we're already reasonably close
    - they're doing some stuff with leaving a >=15-byte literal at the end of
      every block, and maybe other stuff I didn't notice too
        - (somewhat curious to know _why_ this is supposed to be faster)
- if we do get it somewhat close in terms of speed, it would maybe help to
  look at a flamegraph of what's slow (and possibly a corresponding one for rust-snappy)
    - there's some stuff like using `%` instead of `>>` in the hash-table impl
        that maybe matters? (maybe?) -- so it would be good to find out what does/doesn't.
- if we ever get really close to rust-snappy in terms of perf, see their readme
  for benchmarks to try to match them
*/

mod hash_table;

const BLOCK_SIZE: usize = 64 * 1024;
const MAX_COPY_LEN: usize = 64;

/// Panics if input.len() > u32::MAX.
pub fn compress(input: &[u8]) -> Vec<u8> {
    let n = input.len();
    assert!(n <= usize::try_from(u32::MAX).unwrap());

    // // TODO: n isn't actually an upper bound, in the worst case
    // // * maybe look at the formula snappy uses?
    // let mut out = Vec::with_capacity(n);

    // // Header: uncompressed length.
    // varint::write(u32::try_from(n).unwrap(), &mut out);

    // for chunk in input.chunks(BLOCK_SIZE) {
    //     compress_block(chunk, &mut out);
    // }

    let mut header = Vec::with_capacity(5);
    varint::write(u32::try_from(n).unwrap(), &mut header);
    let header = rayon::iter::once(header);

    let output_chunks = input.par_chunks(BLOCK_SIZE).map(|chunk| {
        let mut out_chunk = Vec::with_capacity(BLOCK_SIZE);
        compress_block(chunk, &mut out_chunk);
        out_chunk
    });

    header.chain(output_chunks).flatten().collect()
}

fn compress_block(input: &[u8], out: &mut Vec<u8>) {
    let n = input.len();

    let mut emitted = 0; // num input bytes compressed so far
    let mut seen = HashTable::new(n);

    let mut i = 0;
    let i_limit = n.saturating_sub(3); // exclusive
    while i < i_limit {
        let mut next_i = i + 1;

        // Have we seen this 4-byte pattern before?
        let curr_hash = seen.hash(&input[i..i + 4]);
        let i0 = seen.get(curr_hash);
        if i != 0 && input[i..i + 4] == input[i0..i0 + 4] {
            // Extend the match as much as possible: find the first place where
            // the slices differ. (Note that the slices might overlap, and
            // that's ok.)
            let mut match_len = 4;
            while match_len < MAX_COPY_LEN
                && i + match_len < n
                && input[i0 + match_len] == input[i + match_len]
            {
                match_len += 1;
            }

            if emitted < i {
                literal(&input[emitted..i], out);
                emitted = i;
            }

            copy(i - i0, match_len, out);
            emitted += match_len;

            next_i = min(i + match_len, i_limit);
        }

        // Update `seen`.
        seen.insert(curr_hash, i);
        i += 1;
        while i < next_i {
            let hash = seen.hash(&input[i..i + 4]);
            seen.insert(hash, i);
            i += 1;
        }
    }

    if emitted < n {
        literal(&input[emitted..n], out);
    }
}

fn literal(data: &[u8], out: &mut Vec<u8>) {
    debug_assert!(!data.is_empty());
    let len = data.len();
    debug_assert!(len <= usize::try_from(u32::MAX).unwrap());

    if len <= 60 {
        let len = u8::try_from(len).unwrap();
        let tag = (len - 1) << 2;
        out.push(tag);
    } else {
        let len = u32::try_from(len).unwrap();
        let len_minus_one = len - 1;
        let (width, encoding) = match len_minus_one {
            0x0..=0xff => (1, 60),
            0x100..=0xffff => (2, 61),
            0x1_0000..=0xff_ffff => (3, 62),
            0x100_0000..=0xffff_ffff => (4, 63),
        };

        let tag = encoding << 2;
        out.push(tag);

        let buf = len_minus_one.to_le_bytes();
        out.extend_from_slice(&buf[..width]);
    }

    out.extend_from_slice(data);
}

fn copy(offset: usize, len: usize, out: &mut Vec<u8>) {
    debug_assert!(offset != 0);
    debug_assert!(offset <= usize::try_from(u32::MAX).unwrap());
    debug_assert!(len != 0);
    debug_assert!(len <= MAX_COPY_LEN);

    // "1-byte" offset
    if (4..=11).contains(&len) && offset < 1 << 11 {
        let len_minus_4 = u8::try_from(len - 4).unwrap();
        debug_assert!(len_minus_4 <= 0x7);

        let offset_high = u8::try_from(offset >> 8).unwrap();
        debug_assert!(offset_high <= 0x7);
        let offset_low = u8::try_from(offset & 0xff).unwrap();

        let tag = offset_high << 5 | len_minus_4 << 2 | 0x1;
        out.push(tag);
        out.push(offset_low);
    }
    // 2-byte offset
    else if let Ok(offset) = u16::try_from(offset) {
        let tag = u8::try_from(len - 1).unwrap() << 2 | 0x2;
        out.push(tag);
        out.extend_from_slice(&offset.to_le_bytes());
    }
    // 4-byte offset
    else {
        let offset = u32::try_from(offset).unwrap();
        let tag = u8::try_from(len - 1).unwrap() << 2 | 0x3;
        out.push(tag);
        out.extend_from_slice(&offset.to_le_bytes());
    }
}
