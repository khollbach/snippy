pub fn parse(tag_byte: u8) -> Tag {
    // unpack(LOOKUP_TABLE[usize::from(tag_byte)])
    Tag::parse_slow(tag_byte)
}

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

/// Pre-computed, to speed things up.
const LOOKUP_TABLE: [PackedTag; 256] = lookup_table();

#[derive(Debug, Clone, Copy)]
struct PackedTag {
    /// Layout:
    /// saaa_0bbb_cccc_cccc
    ///
    /// `s` is 0 for copy, and 1 for literal.
    /// `a` is offset_num_bytes for copy (and 0 for literal).
    /// `b` is offset_high_bits for copy, and length_num_bytes for literal.
    /// `c` is len for copy, and length_value for literal (whenever length_num_bytes is 0).
    bits: u16,
}

const fn pack(tag: Tag) -> PackedTag {
    let bits = match tag {
        Tag::Literal(tag) => match tag {
            LiteralTag::LengthValue(len) => 0x8000 | len as u16,
            LiteralTag::LengthNumBytes(b) => 0x8000 | (b as u16) << 8,
        },
        Tag::Copy(tag) => {
            (tag.offset_num_bytes as u16) << 12 | tag.offset_high_bits | tag.len as u16
        }
    };
    PackedTag { bits }
}

const fn unpack(packed: PackedTag) -> Tag {
    if packed.bits & 0x8000 != 0 {
        let length_num_bytes = packed.bits >> 8 & 0x07;
        if length_num_bytes != 0 {
            Tag::Literal(LiteralTag::LengthNumBytes(length_num_bytes as u8))
        } else {
            let len = packed.bits & 0xff;
            Tag::Literal(LiteralTag::LengthValue(len as u8))
        }
    } else {
        Tag::Copy(CopyTag {
            len: (packed.bits & 0xff) as u8,
            offset_high_bits: packed.bits & 0x0700,
            offset_num_bytes: (packed.bits >> 12) as u8,
        })
    }
}

impl Tag {
    const fn parse_slow(tag: u8) -> Self {
        if tag & 0x3 == 0x0 {
            Self::Literal(LiteralTag::parse_slow(tag))
        } else {
            Self::Copy(CopyTag::parse_slow(tag))
        }
    }
}

impl LiteralTag {
    const fn parse_slow(tag: u8) -> Self {
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
    const fn parse_slow(tag: u8) -> Self {
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

const fn lookup_table() -> [PackedTag; 256] {
    let mut table = [PackedTag { bits: 0 }; 256];

    // TODO: _surely_ there's a better way...
    table[0] = pack(Tag::parse_slow(0));
    table[1] = pack(Tag::parse_slow(1));
    table[2] = pack(Tag::parse_slow(2));
    table[3] = pack(Tag::parse_slow(3));
    table[4] = pack(Tag::parse_slow(4));
    table[5] = pack(Tag::parse_slow(5));
    table[6] = pack(Tag::parse_slow(6));
    table[7] = pack(Tag::parse_slow(7));
    table[8] = pack(Tag::parse_slow(8));
    table[9] = pack(Tag::parse_slow(9));
    table[10] = pack(Tag::parse_slow(10));
    table[11] = pack(Tag::parse_slow(11));
    table[12] = pack(Tag::parse_slow(12));
    table[13] = pack(Tag::parse_slow(13));
    table[14] = pack(Tag::parse_slow(14));
    table[15] = pack(Tag::parse_slow(15));
    table[16] = pack(Tag::parse_slow(16));
    table[17] = pack(Tag::parse_slow(17));
    table[18] = pack(Tag::parse_slow(18));
    table[19] = pack(Tag::parse_slow(19));
    table[20] = pack(Tag::parse_slow(20));
    table[21] = pack(Tag::parse_slow(21));
    table[22] = pack(Tag::parse_slow(22));
    table[23] = pack(Tag::parse_slow(23));
    table[24] = pack(Tag::parse_slow(24));
    table[25] = pack(Tag::parse_slow(25));
    table[26] = pack(Tag::parse_slow(26));
    table[27] = pack(Tag::parse_slow(27));
    table[28] = pack(Tag::parse_slow(28));
    table[29] = pack(Tag::parse_slow(29));
    table[30] = pack(Tag::parse_slow(30));
    table[31] = pack(Tag::parse_slow(31));
    table[32] = pack(Tag::parse_slow(32));
    table[33] = pack(Tag::parse_slow(33));
    table[34] = pack(Tag::parse_slow(34));
    table[35] = pack(Tag::parse_slow(35));
    table[36] = pack(Tag::parse_slow(36));
    table[37] = pack(Tag::parse_slow(37));
    table[38] = pack(Tag::parse_slow(38));
    table[39] = pack(Tag::parse_slow(39));
    table[40] = pack(Tag::parse_slow(40));
    table[41] = pack(Tag::parse_slow(41));
    table[42] = pack(Tag::parse_slow(42));
    table[43] = pack(Tag::parse_slow(43));
    table[44] = pack(Tag::parse_slow(44));
    table[45] = pack(Tag::parse_slow(45));
    table[46] = pack(Tag::parse_slow(46));
    table[47] = pack(Tag::parse_slow(47));
    table[48] = pack(Tag::parse_slow(48));
    table[49] = pack(Tag::parse_slow(49));
    table[50] = pack(Tag::parse_slow(50));
    table[51] = pack(Tag::parse_slow(51));
    table[52] = pack(Tag::parse_slow(52));
    table[53] = pack(Tag::parse_slow(53));
    table[54] = pack(Tag::parse_slow(54));
    table[55] = pack(Tag::parse_slow(55));
    table[56] = pack(Tag::parse_slow(56));
    table[57] = pack(Tag::parse_slow(57));
    table[58] = pack(Tag::parse_slow(58));
    table[59] = pack(Tag::parse_slow(59));
    table[60] = pack(Tag::parse_slow(60));
    table[61] = pack(Tag::parse_slow(61));
    table[62] = pack(Tag::parse_slow(62));
    table[63] = pack(Tag::parse_slow(63));
    table[64] = pack(Tag::parse_slow(64));
    table[65] = pack(Tag::parse_slow(65));
    table[66] = pack(Tag::parse_slow(66));
    table[67] = pack(Tag::parse_slow(67));
    table[68] = pack(Tag::parse_slow(68));
    table[69] = pack(Tag::parse_slow(69));
    table[70] = pack(Tag::parse_slow(70));
    table[71] = pack(Tag::parse_slow(71));
    table[72] = pack(Tag::parse_slow(72));
    table[73] = pack(Tag::parse_slow(73));
    table[74] = pack(Tag::parse_slow(74));
    table[75] = pack(Tag::parse_slow(75));
    table[76] = pack(Tag::parse_slow(76));
    table[77] = pack(Tag::parse_slow(77));
    table[78] = pack(Tag::parse_slow(78));
    table[79] = pack(Tag::parse_slow(79));
    table[80] = pack(Tag::parse_slow(80));
    table[81] = pack(Tag::parse_slow(81));
    table[82] = pack(Tag::parse_slow(82));
    table[83] = pack(Tag::parse_slow(83));
    table[84] = pack(Tag::parse_slow(84));
    table[85] = pack(Tag::parse_slow(85));
    table[86] = pack(Tag::parse_slow(86));
    table[87] = pack(Tag::parse_slow(87));
    table[88] = pack(Tag::parse_slow(88));
    table[89] = pack(Tag::parse_slow(89));
    table[90] = pack(Tag::parse_slow(90));
    table[91] = pack(Tag::parse_slow(91));
    table[92] = pack(Tag::parse_slow(92));
    table[93] = pack(Tag::parse_slow(93));
    table[94] = pack(Tag::parse_slow(94));
    table[95] = pack(Tag::parse_slow(95));
    table[96] = pack(Tag::parse_slow(96));
    table[97] = pack(Tag::parse_slow(97));
    table[98] = pack(Tag::parse_slow(98));
    table[99] = pack(Tag::parse_slow(99));
    table[100] = pack(Tag::parse_slow(100));
    table[101] = pack(Tag::parse_slow(101));
    table[102] = pack(Tag::parse_slow(102));
    table[103] = pack(Tag::parse_slow(103));
    table[104] = pack(Tag::parse_slow(104));
    table[105] = pack(Tag::parse_slow(105));
    table[106] = pack(Tag::parse_slow(106));
    table[107] = pack(Tag::parse_slow(107));
    table[108] = pack(Tag::parse_slow(108));
    table[109] = pack(Tag::parse_slow(109));
    table[110] = pack(Tag::parse_slow(110));
    table[111] = pack(Tag::parse_slow(111));
    table[112] = pack(Tag::parse_slow(112));
    table[113] = pack(Tag::parse_slow(113));
    table[114] = pack(Tag::parse_slow(114));
    table[115] = pack(Tag::parse_slow(115));
    table[116] = pack(Tag::parse_slow(116));
    table[117] = pack(Tag::parse_slow(117));
    table[118] = pack(Tag::parse_slow(118));
    table[119] = pack(Tag::parse_slow(119));
    table[120] = pack(Tag::parse_slow(120));
    table[121] = pack(Tag::parse_slow(121));
    table[122] = pack(Tag::parse_slow(122));
    table[123] = pack(Tag::parse_slow(123));
    table[124] = pack(Tag::parse_slow(124));
    table[125] = pack(Tag::parse_slow(125));
    table[126] = pack(Tag::parse_slow(126));
    table[127] = pack(Tag::parse_slow(127));
    table[128] = pack(Tag::parse_slow(128));
    table[129] = pack(Tag::parse_slow(129));
    table[130] = pack(Tag::parse_slow(130));
    table[131] = pack(Tag::parse_slow(131));
    table[132] = pack(Tag::parse_slow(132));
    table[133] = pack(Tag::parse_slow(133));
    table[134] = pack(Tag::parse_slow(134));
    table[135] = pack(Tag::parse_slow(135));
    table[136] = pack(Tag::parse_slow(136));
    table[137] = pack(Tag::parse_slow(137));
    table[138] = pack(Tag::parse_slow(138));
    table[139] = pack(Tag::parse_slow(139));
    table[140] = pack(Tag::parse_slow(140));
    table[141] = pack(Tag::parse_slow(141));
    table[142] = pack(Tag::parse_slow(142));
    table[143] = pack(Tag::parse_slow(143));
    table[144] = pack(Tag::parse_slow(144));
    table[145] = pack(Tag::parse_slow(145));
    table[146] = pack(Tag::parse_slow(146));
    table[147] = pack(Tag::parse_slow(147));
    table[148] = pack(Tag::parse_slow(148));
    table[149] = pack(Tag::parse_slow(149));
    table[150] = pack(Tag::parse_slow(150));
    table[151] = pack(Tag::parse_slow(151));
    table[152] = pack(Tag::parse_slow(152));
    table[153] = pack(Tag::parse_slow(153));
    table[154] = pack(Tag::parse_slow(154));
    table[155] = pack(Tag::parse_slow(155));
    table[156] = pack(Tag::parse_slow(156));
    table[157] = pack(Tag::parse_slow(157));
    table[158] = pack(Tag::parse_slow(158));
    table[159] = pack(Tag::parse_slow(159));
    table[160] = pack(Tag::parse_slow(160));
    table[161] = pack(Tag::parse_slow(161));
    table[162] = pack(Tag::parse_slow(162));
    table[163] = pack(Tag::parse_slow(163));
    table[164] = pack(Tag::parse_slow(164));
    table[165] = pack(Tag::parse_slow(165));
    table[166] = pack(Tag::parse_slow(166));
    table[167] = pack(Tag::parse_slow(167));
    table[168] = pack(Tag::parse_slow(168));
    table[169] = pack(Tag::parse_slow(169));
    table[170] = pack(Tag::parse_slow(170));
    table[171] = pack(Tag::parse_slow(171));
    table[172] = pack(Tag::parse_slow(172));
    table[173] = pack(Tag::parse_slow(173));
    table[174] = pack(Tag::parse_slow(174));
    table[175] = pack(Tag::parse_slow(175));
    table[176] = pack(Tag::parse_slow(176));
    table[177] = pack(Tag::parse_slow(177));
    table[178] = pack(Tag::parse_slow(178));
    table[179] = pack(Tag::parse_slow(179));
    table[180] = pack(Tag::parse_slow(180));
    table[181] = pack(Tag::parse_slow(181));
    table[182] = pack(Tag::parse_slow(182));
    table[183] = pack(Tag::parse_slow(183));
    table[184] = pack(Tag::parse_slow(184));
    table[185] = pack(Tag::parse_slow(185));
    table[186] = pack(Tag::parse_slow(186));
    table[187] = pack(Tag::parse_slow(187));
    table[188] = pack(Tag::parse_slow(188));
    table[189] = pack(Tag::parse_slow(189));
    table[190] = pack(Tag::parse_slow(190));
    table[191] = pack(Tag::parse_slow(191));
    table[192] = pack(Tag::parse_slow(192));
    table[193] = pack(Tag::parse_slow(193));
    table[194] = pack(Tag::parse_slow(194));
    table[195] = pack(Tag::parse_slow(195));
    table[196] = pack(Tag::parse_slow(196));
    table[197] = pack(Tag::parse_slow(197));
    table[198] = pack(Tag::parse_slow(198));
    table[199] = pack(Tag::parse_slow(199));
    table[200] = pack(Tag::parse_slow(200));
    table[201] = pack(Tag::parse_slow(201));
    table[202] = pack(Tag::parse_slow(202));
    table[203] = pack(Tag::parse_slow(203));
    table[204] = pack(Tag::parse_slow(204));
    table[205] = pack(Tag::parse_slow(205));
    table[206] = pack(Tag::parse_slow(206));
    table[207] = pack(Tag::parse_slow(207));
    table[208] = pack(Tag::parse_slow(208));
    table[209] = pack(Tag::parse_slow(209));
    table[210] = pack(Tag::parse_slow(210));
    table[211] = pack(Tag::parse_slow(211));
    table[212] = pack(Tag::parse_slow(212));
    table[213] = pack(Tag::parse_slow(213));
    table[214] = pack(Tag::parse_slow(214));
    table[215] = pack(Tag::parse_slow(215));
    table[216] = pack(Tag::parse_slow(216));
    table[217] = pack(Tag::parse_slow(217));
    table[218] = pack(Tag::parse_slow(218));
    table[219] = pack(Tag::parse_slow(219));
    table[220] = pack(Tag::parse_slow(220));
    table[221] = pack(Tag::parse_slow(221));
    table[222] = pack(Tag::parse_slow(222));
    table[223] = pack(Tag::parse_slow(223));
    table[224] = pack(Tag::parse_slow(224));
    table[225] = pack(Tag::parse_slow(225));
    table[226] = pack(Tag::parse_slow(226));
    table[227] = pack(Tag::parse_slow(227));
    table[228] = pack(Tag::parse_slow(228));
    table[229] = pack(Tag::parse_slow(229));
    table[230] = pack(Tag::parse_slow(230));
    table[231] = pack(Tag::parse_slow(231));
    table[232] = pack(Tag::parse_slow(232));
    table[233] = pack(Tag::parse_slow(233));
    table[234] = pack(Tag::parse_slow(234));
    table[235] = pack(Tag::parse_slow(235));
    table[236] = pack(Tag::parse_slow(236));
    table[237] = pack(Tag::parse_slow(237));
    table[238] = pack(Tag::parse_slow(238));
    table[239] = pack(Tag::parse_slow(239));
    table[240] = pack(Tag::parse_slow(240));
    table[241] = pack(Tag::parse_slow(241));
    table[242] = pack(Tag::parse_slow(242));
    table[243] = pack(Tag::parse_slow(243));
    table[244] = pack(Tag::parse_slow(244));
    table[245] = pack(Tag::parse_slow(245));
    table[246] = pack(Tag::parse_slow(246));
    table[247] = pack(Tag::parse_slow(247));
    table[248] = pack(Tag::parse_slow(248));
    table[249] = pack(Tag::parse_slow(249));
    table[250] = pack(Tag::parse_slow(250));
    table[251] = pack(Tag::parse_slow(251));
    table[252] = pack(Tag::parse_slow(252));
    table[253] = pack(Tag::parse_slow(253));
    table[254] = pack(Tag::parse_slow(254));
    table[255] = pack(Tag::parse_slow(255));

    table
}
