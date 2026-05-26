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
    pub type_: CopyType,
}

#[derive(Debug, Clone, Copy)]
pub enum CopyType {
    OneByteOffset {
        high_bits: u8,
    },
    /// Either 2 or 4.
    ManyByteOffset {
        width: u8,
    },
}

/// Pre-computed, to speed things up.
pub const LOOKUP_TABLE: [Tag; 256] = lookup_table();

impl Tag {
    const fn parse(tag: u8) -> Self {
        if tag & 0x3 == 0x0 {
            Self::Literal(LiteralTag::parse(tag))
        } else {
            Self::Copy(CopyTag::parse(tag))
        }
    }
}

impl LiteralTag {
    const fn parse(tag: u8) -> Self {
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
    const fn parse(tag: u8) -> Self {
        match tag & 0x3 {
            0 => panic!("cannot parse literal tag byte as copy tag"),
            1 => Self {
                len: (tag >> 2 & 0x7) + 4, // middle 3 bits
                type_: CopyType::OneByteOffset {
                    high_bits: tag >> 5, // high 3 bits
                },
            },
            2 => Self {
                len: (tag >> 2) + 1,
                type_: CopyType::ManyByteOffset { width: 2 },
            },
            3 => Self {
                len: (tag >> 2) + 1,
                type_: CopyType::ManyByteOffset { width: 4 },
            },
            _ => unreachable!(),
        }
    }
}

const fn lookup_table() -> [Tag; 256] {
    let mut table = [Tag::Literal(LiteralTag::LengthValue(0)); 256];

    // TODO: _surely_ there's a better way...
    table[0] = Tag::parse(0);
    table[1] = Tag::parse(1);
    table[2] = Tag::parse(2);
    table[3] = Tag::parse(3);
    table[4] = Tag::parse(4);
    table[5] = Tag::parse(5);
    table[6] = Tag::parse(6);
    table[7] = Tag::parse(7);
    table[8] = Tag::parse(8);
    table[9] = Tag::parse(9);
    table[10] = Tag::parse(10);
    table[11] = Tag::parse(11);
    table[12] = Tag::parse(12);
    table[13] = Tag::parse(13);
    table[14] = Tag::parse(14);
    table[15] = Tag::parse(15);
    table[16] = Tag::parse(16);
    table[17] = Tag::parse(17);
    table[18] = Tag::parse(18);
    table[19] = Tag::parse(19);
    table[20] = Tag::parse(20);
    table[21] = Tag::parse(21);
    table[22] = Tag::parse(22);
    table[23] = Tag::parse(23);
    table[24] = Tag::parse(24);
    table[25] = Tag::parse(25);
    table[26] = Tag::parse(26);
    table[27] = Tag::parse(27);
    table[28] = Tag::parse(28);
    table[29] = Tag::parse(29);
    table[30] = Tag::parse(30);
    table[31] = Tag::parse(31);
    table[32] = Tag::parse(32);
    table[33] = Tag::parse(33);
    table[34] = Tag::parse(34);
    table[35] = Tag::parse(35);
    table[36] = Tag::parse(36);
    table[37] = Tag::parse(37);
    table[38] = Tag::parse(38);
    table[39] = Tag::parse(39);
    table[40] = Tag::parse(40);
    table[41] = Tag::parse(41);
    table[42] = Tag::parse(42);
    table[43] = Tag::parse(43);
    table[44] = Tag::parse(44);
    table[45] = Tag::parse(45);
    table[46] = Tag::parse(46);
    table[47] = Tag::parse(47);
    table[48] = Tag::parse(48);
    table[49] = Tag::parse(49);
    table[50] = Tag::parse(50);
    table[51] = Tag::parse(51);
    table[52] = Tag::parse(52);
    table[53] = Tag::parse(53);
    table[54] = Tag::parse(54);
    table[55] = Tag::parse(55);
    table[56] = Tag::parse(56);
    table[57] = Tag::parse(57);
    table[58] = Tag::parse(58);
    table[59] = Tag::parse(59);
    table[60] = Tag::parse(60);
    table[61] = Tag::parse(61);
    table[62] = Tag::parse(62);
    table[63] = Tag::parse(63);
    table[64] = Tag::parse(64);
    table[65] = Tag::parse(65);
    table[66] = Tag::parse(66);
    table[67] = Tag::parse(67);
    table[68] = Tag::parse(68);
    table[69] = Tag::parse(69);
    table[70] = Tag::parse(70);
    table[71] = Tag::parse(71);
    table[72] = Tag::parse(72);
    table[73] = Tag::parse(73);
    table[74] = Tag::parse(74);
    table[75] = Tag::parse(75);
    table[76] = Tag::parse(76);
    table[77] = Tag::parse(77);
    table[78] = Tag::parse(78);
    table[79] = Tag::parse(79);
    table[80] = Tag::parse(80);
    table[81] = Tag::parse(81);
    table[82] = Tag::parse(82);
    table[83] = Tag::parse(83);
    table[84] = Tag::parse(84);
    table[85] = Tag::parse(85);
    table[86] = Tag::parse(86);
    table[87] = Tag::parse(87);
    table[88] = Tag::parse(88);
    table[89] = Tag::parse(89);
    table[90] = Tag::parse(90);
    table[91] = Tag::parse(91);
    table[92] = Tag::parse(92);
    table[93] = Tag::parse(93);
    table[94] = Tag::parse(94);
    table[95] = Tag::parse(95);
    table[96] = Tag::parse(96);
    table[97] = Tag::parse(97);
    table[98] = Tag::parse(98);
    table[99] = Tag::parse(99);
    table[100] = Tag::parse(100);
    table[101] = Tag::parse(101);
    table[102] = Tag::parse(102);
    table[103] = Tag::parse(103);
    table[104] = Tag::parse(104);
    table[105] = Tag::parse(105);
    table[106] = Tag::parse(106);
    table[107] = Tag::parse(107);
    table[108] = Tag::parse(108);
    table[109] = Tag::parse(109);
    table[110] = Tag::parse(110);
    table[111] = Tag::parse(111);
    table[112] = Tag::parse(112);
    table[113] = Tag::parse(113);
    table[114] = Tag::parse(114);
    table[115] = Tag::parse(115);
    table[116] = Tag::parse(116);
    table[117] = Tag::parse(117);
    table[118] = Tag::parse(118);
    table[119] = Tag::parse(119);
    table[120] = Tag::parse(120);
    table[121] = Tag::parse(121);
    table[122] = Tag::parse(122);
    table[123] = Tag::parse(123);
    table[124] = Tag::parse(124);
    table[125] = Tag::parse(125);
    table[126] = Tag::parse(126);
    table[127] = Tag::parse(127);
    table[128] = Tag::parse(128);
    table[129] = Tag::parse(129);
    table[130] = Tag::parse(130);
    table[131] = Tag::parse(131);
    table[132] = Tag::parse(132);
    table[133] = Tag::parse(133);
    table[134] = Tag::parse(134);
    table[135] = Tag::parse(135);
    table[136] = Tag::parse(136);
    table[137] = Tag::parse(137);
    table[138] = Tag::parse(138);
    table[139] = Tag::parse(139);
    table[140] = Tag::parse(140);
    table[141] = Tag::parse(141);
    table[142] = Tag::parse(142);
    table[143] = Tag::parse(143);
    table[144] = Tag::parse(144);
    table[145] = Tag::parse(145);
    table[146] = Tag::parse(146);
    table[147] = Tag::parse(147);
    table[148] = Tag::parse(148);
    table[149] = Tag::parse(149);
    table[150] = Tag::parse(150);
    table[151] = Tag::parse(151);
    table[152] = Tag::parse(152);
    table[153] = Tag::parse(153);
    table[154] = Tag::parse(154);
    table[155] = Tag::parse(155);
    table[156] = Tag::parse(156);
    table[157] = Tag::parse(157);
    table[158] = Tag::parse(158);
    table[159] = Tag::parse(159);
    table[160] = Tag::parse(160);
    table[161] = Tag::parse(161);
    table[162] = Tag::parse(162);
    table[163] = Tag::parse(163);
    table[164] = Tag::parse(164);
    table[165] = Tag::parse(165);
    table[166] = Tag::parse(166);
    table[167] = Tag::parse(167);
    table[168] = Tag::parse(168);
    table[169] = Tag::parse(169);
    table[170] = Tag::parse(170);
    table[171] = Tag::parse(171);
    table[172] = Tag::parse(172);
    table[173] = Tag::parse(173);
    table[174] = Tag::parse(174);
    table[175] = Tag::parse(175);
    table[176] = Tag::parse(176);
    table[177] = Tag::parse(177);
    table[178] = Tag::parse(178);
    table[179] = Tag::parse(179);
    table[180] = Tag::parse(180);
    table[181] = Tag::parse(181);
    table[182] = Tag::parse(182);
    table[183] = Tag::parse(183);
    table[184] = Tag::parse(184);
    table[185] = Tag::parse(185);
    table[186] = Tag::parse(186);
    table[187] = Tag::parse(187);
    table[188] = Tag::parse(188);
    table[189] = Tag::parse(189);
    table[190] = Tag::parse(190);
    table[191] = Tag::parse(191);
    table[192] = Tag::parse(192);
    table[193] = Tag::parse(193);
    table[194] = Tag::parse(194);
    table[195] = Tag::parse(195);
    table[196] = Tag::parse(196);
    table[197] = Tag::parse(197);
    table[198] = Tag::parse(198);
    table[199] = Tag::parse(199);
    table[200] = Tag::parse(200);
    table[201] = Tag::parse(201);
    table[202] = Tag::parse(202);
    table[203] = Tag::parse(203);
    table[204] = Tag::parse(204);
    table[205] = Tag::parse(205);
    table[206] = Tag::parse(206);
    table[207] = Tag::parse(207);
    table[208] = Tag::parse(208);
    table[209] = Tag::parse(209);
    table[210] = Tag::parse(210);
    table[211] = Tag::parse(211);
    table[212] = Tag::parse(212);
    table[213] = Tag::parse(213);
    table[214] = Tag::parse(214);
    table[215] = Tag::parse(215);
    table[216] = Tag::parse(216);
    table[217] = Tag::parse(217);
    table[218] = Tag::parse(218);
    table[219] = Tag::parse(219);
    table[220] = Tag::parse(220);
    table[221] = Tag::parse(221);
    table[222] = Tag::parse(222);
    table[223] = Tag::parse(223);
    table[224] = Tag::parse(224);
    table[225] = Tag::parse(225);
    table[226] = Tag::parse(226);
    table[227] = Tag::parse(227);
    table[228] = Tag::parse(228);
    table[229] = Tag::parse(229);
    table[230] = Tag::parse(230);
    table[231] = Tag::parse(231);
    table[232] = Tag::parse(232);
    table[233] = Tag::parse(233);
    table[234] = Tag::parse(234);
    table[235] = Tag::parse(235);
    table[236] = Tag::parse(236);
    table[237] = Tag::parse(237);
    table[238] = Tag::parse(238);
    table[239] = Tag::parse(239);
    table[240] = Tag::parse(240);
    table[241] = Tag::parse(241);
    table[242] = Tag::parse(242);
    table[243] = Tag::parse(243);
    table[244] = Tag::parse(244);
    table[245] = Tag::parse(245);
    table[246] = Tag::parse(246);
    table[247] = Tag::parse(247);
    table[248] = Tag::parse(248);
    table[249] = Tag::parse(249);
    table[250] = Tag::parse(250);
    table[251] = Tag::parse(251);
    table[252] = Tag::parse(252);
    table[253] = Tag::parse(253);
    table[254] = Tag::parse(254);
    table[255] = Tag::parse(255);

    table
}
