#[derive(Debug, Clone, Copy)]
pub enum Tag {
    Literal(LiteralTag),
    Copy(CopyTag),
}

#[derive(Debug, Clone, Copy)]
pub enum LiteralTag {
    /// 1..=60
    LengthValue(u8),
    /// 1..=4
    LengthNumBytes(u8),
}

#[derive(Debug, Clone, Copy)]
pub struct CopyTag {
    pub len: u8,
    /// Has this pattern of bits:
    /// xxxx_xbbb_xxxx_xxxx
    /// Gets or'd with offset.
    pub offset_high_bits: u16,
    /// 1, 2, or 4.
    pub offset_num_bytes: u8,
}

impl Tag {
    pub fn parse(tag: u8) -> Self {
        if tag & 0x3 == 0x0 {
            Self::Literal(LiteralTag::parse(tag))
        } else {
            Self::Copy(CopyTag::parse(tag))
        }
    }
}

impl LiteralTag {
    fn parse(tag: u8) -> Self {
        match tag >> 2 {
            60 => Self::LengthNumBytes(1),
            61 => Self::LengthNumBytes(2),
            62 => Self::LengthNumBytes(3),
            63 => Self::LengthNumBytes(4),
            n => Self::LengthValue(n + 1),
        }
    }
}

impl CopyTag {
    fn parse(tag: u8) -> Self {
        match tag & 0x3 {
            0 => panic!("cannot parse literal tag byte as copy tag"),
            1 => Self {
                // middle 3 bits
                len: (tag >> 2 & 0x7) + 4,
                // high 3 bits
                offset_high_bits: ((tag >> 5) as u16) << 8,
                offset_num_bytes: 1,
            },
            2 => Self {
                len: (tag >> 2) + 1,
                offset_high_bits: 0,
                offset_num_bytes: 2,
            },
            3 => Self {
                len: (tag >> 2) + 1,
                offset_high_bits: 0,
                offset_num_bytes: 4,
            },
            _ => unreachable!(),
        }
    }
}
