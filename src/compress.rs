use std::{cmp::min, collections::HashMap};

use crate::varint;

const MAX_COPY_LEN: usize = 64;

/// Panics if input.len() > u32::MAX.
pub fn compress(input: &[u8]) -> Vec<u8> {
    let n = input.len();
    assert!(n <= usize::try_from(u32::MAX).unwrap());

    // TODO: n isn't actually an upper bound, in the worst case
    // * maybe look at the formula snappy uses?
    let mut out = Vec::with_capacity(n);

    // Header: uncompressed length.
    varint::write(u32::try_from(n).unwrap(), &mut out);

    let mut emitted = 0; // num input bytes compressed so far
    let mut seen = HashMap::with_capacity(n); // (todo...)

    let mut i = 0;
    let i_limit = n.saturating_sub(3); // exclusive
    while i < i_limit {
        let mut next_i = i + 1;

        // It's a match!
        if let Some(&i0) = seen.get(&input[i..i + 4]) {
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
                literal(&input[emitted..i], &mut out);
                emitted = i;
            }

            copy(i - i0, match_len, &mut out);
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
