use crate::DrawHandle;

pub struct TextRenderer {
    bitmap: &'static [u8],
    bytes_per_glyph: u32,
    width: u32,
    height: u32,
}

macro_rules! read_u32 {
    ($ptr: expr) => {{
        let Some((int_bytes, rest)) = $ptr.split_first_chunk::<4>() else {
            panic!("unexpected end of file");
        };
        $ptr = rest;
        u32::from_le_bytes(*int_bytes)
    }};
}

macro_rules! assert_field_eq {
    ($var: expr, $exp: expr) => {
        if $var != $exp {
            panic!(concat!(
                "unexpected ",
                stringify!($var),
                ", expected ",
                stringify!($exp)
            ));
        }
    };
}

impl TextRenderer {
    pub const fn new() -> Self {
        const FONT: &[u8] = include_bytes!("../Tamsyn8x16r.psf");
        const PSF2_MAGIC: u32 = 0x86_4a_b5_72;

        let mut ptr = FONT;

        let magic = read_u32!(ptr);
        let version = read_u32!(ptr);
        let header_size = read_u32!(ptr);
        let _flags = read_u32!(ptr);
        let _num_glyphs = read_u32!(ptr);
        let bytes_per_glyph = read_u32!(ptr);
        let height = read_u32!(ptr);
        let width = read_u32!(ptr);
        let bitmap = ptr;

        assert_field_eq!(magic, PSF2_MAGIC);
        assert_field_eq!(version, 0);
        assert_field_eq!(header_size, 32);

        Self {
            bitmap,
            bytes_per_glyph,
            width,
            height,
        }
    }

    pub fn render(&self, d: &mut DrawHandle, mut pos_x: u32, pos_y: u32, color: u32, text: &str) {
        for ch in text.chars() {
            let ascii = if ch.is_ascii() { ch } else { '?' };

            self.render_char(d, pos_x, pos_y, color, ascii);

            pos_x += self.width;
        }
    }

    fn render_char(&self, d: &mut DrawHandle, pos_x: u32, pos_y: u32, color: u32, ascii: char) {
        let bpg = self.bytes_per_glyph as usize;
        let bytes_per_row = self.height as usize / bpg;
        let idx = ascii as usize * bpg;
        let char_bitmap = &self.bitmap[idx..idx + bpg];

        for row in 0..self.height {
            for byte in 0..bytes_per_row {
                let glypth_byte = char_bitmap[row as usize + byte];

                if glypth_byte == 0 {
                    continue;
                }

                for bit in 0..8 {
                    if glypth_byte & (1 << bit) != 0 {
                        d.set(pos_x + 7 - bit, pos_y + row, color);
                    }
                }
            }
        }
    }
}
