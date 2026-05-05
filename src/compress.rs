use std::{cmp::min, collections::HashMap};

use crate::varint;

/// Panics if input.len() > u32::MAX.
pub fn compress(input: &[u8]) -> Vec<u8> {
    let n = input.len();
    assert!(n <= usize::try_from(u32::MAX).unwrap());

    // TODO: n isn't actually an upper bound, in the worst case
    let mut out = Vec::with_capacity(n);

    // Header: uncompressed length.
    varint::write(u32::try_from(n).unwrap(), &mut out);

    let mut emitted = 0; // num input bytes compressed so far
    let mut seen = HashMap::new();

    let mut i = 0;
    let i_limit = n - 3; // exclusive
    while i < i_limit {
        let mut next_i = i + 1;

        // It's a match!
        if let Some(&i0) = seen.get(&input[i..i + 4]) {
            // Extend the match as much as possible: find the first place where
            // the slices differ. (Note that the slices might overlap, and
            // that's ok.)
            let mut match_len = 4;
            while i + match_len < n && input[i0 + match_len] == input[i + match_len] {
                match_len += 1;
            }

            if emitted < i {
                literal(&input[emitted..i], &mut out);
                emitted = i;
            }

            // copy(i - i0, match_len, &mut out);
            literal(&input[i0..i0 + match_len], &mut out); // todo: emit copy instead
            emitted += match_len;

            next_i = min(i + match_len, i_limit);
        }

        while i < next_i {
            seen.insert(&input[i..i + 4], i);
            i += 1;
        }
    }

    if emitted < n {
        literal(&input[emitted..n], &mut out);
    }

    out
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
    // todo
    println!("copy offset:{} match_len:{}", offset, len);
}
