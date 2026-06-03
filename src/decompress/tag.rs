use std::io::Read;

use anyhow::{Context, Result};

#[derive(Debug, Clone, Copy)]
pub enum Tag {
    Literal { len: u32 },
    Copy { offset: u32, len: u8 },
}

pub fn read<R: Read>(r: &mut R) -> Result<Tag> {
    let mut buf = [0];
    r.read_exact(&mut buf).context("unexpected EOF")?;
    let tag_byte = buf[0];

    match parse_tag_byte(tag_byte) {
        TagByte::LiteralLengthValue(len) => Ok(Tag::Literal { len: len.into() }),
        TagByte::LiteralLengthNumBytes(n) => {
            let mut buf = [0; 4];
            r.read_exact(&mut buf[..n.into()])
                .context("unexpected EOF")?;

            let len = u32::from_le_bytes(buf)
                .checked_add(1)
                .context("literal len must not equal 2^32 (since total len must not equal 2^32)")?;
            Ok(Tag::Literal { len })
        }
        TagByte::Copy {
            len,
            offset_high_bits,
            offset_num_bytes,
        } => {
            let mut buf = [0; 4];
            r.read_exact(&mut buf[..offset_num_bytes.into()])
                .context("unexpected EOF")?;

            let mut offset = u32::from_le_bytes(buf);
            offset |= offset_high_bits;

            Ok(Tag::Copy { offset, len })
        }
    }
}

enum TagByte {
    /// 1..=60
    LiteralLengthValue(u8),
    /// 1..=4
    LiteralLengthNumBytes(u8),
    Copy {
        /// 1..=64
        len: u8,
        /// Has this pattern of bits:
        /// xxxx_xxxx_xxxx_xxxx__xxxx_xbbb_xxxx_xxxx
        /// Gets or'd with offset.
        offset_high_bits: u32,
        /// 1, 2, or 4.
        offset_num_bytes: u8,
    },
}

fn parse_tag_byte(tag_byte: u8) -> TagByte {
    match tag_byte & 0x3 {
        0 => match tag_byte >> 2 {
            60 => TagByte::LiteralLengthNumBytes(1),
            61 => TagByte::LiteralLengthNumBytes(2),
            62 => TagByte::LiteralLengthNumBytes(3),
            63 => TagByte::LiteralLengthNumBytes(4),
            n => TagByte::LiteralLengthValue(n + 1),
        },
        1 => TagByte::Copy {
            // middle 3 bits
            len: ((tag_byte >> 2) & 0x7) + 4,
            // high 3 bits
            offset_high_bits: u32::from(tag_byte >> 5) << 8,
            offset_num_bytes: 1,
        },
        2 => TagByte::Copy {
            len: (tag_byte >> 2) + 1,
            offset_high_bits: 0,
            offset_num_bytes: 2,
        },
        3 => TagByte::Copy {
            len: (tag_byte >> 2) + 1,
            offset_high_bits: 0,
            offset_num_bytes: 4,
        },
        _ => unreachable!(),
    }
}
