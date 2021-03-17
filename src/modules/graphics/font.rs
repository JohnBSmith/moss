
#[allow(dead_code)]
struct ImgData<'a> {
    w: u32, h: u32, max: u32,
    data: &'a [u8]
}

pub struct GlyphData {
    pub width: u32, pub height: u32,
    pub data: Vec<u8>
}

fn advance_space(p: &[u8], index: &mut usize) {
    let mut i = *index;
    loop {
        let c = p[i];
        if let b' ' | b'\n' | b'\t' | b'\r' = c {i += 1;}
        else {break;}
    }
    *index = i;
}

fn read_uint(p: &[u8], j: &mut usize) -> u32 {
    let mut i = *j;
    let mut value: u32 = 0;
    loop {
        let c = p[i];
        if b'0' <= c && c <= b'9' {
            value = 10*value + (c as u32) - (b'0' as u32);
            i += 1;
        }
        else {break;}
    }
    *j = i;
    value
}

pub enum Fmt {PGM, PPM}

fn pnm_as_data(p: &[u8], fmt: Fmt) -> ImgData {
    match fmt {
        Fmt::PGM => {if &p[0..2] != b"P5" {panic!("Error: not a PGM");}},
        Fmt::PPM => {if &p[0..2] != b"P6" {panic!("Error: not a PPM");}}
    }
    let mut i = 2;
    advance_space(p, &mut i);
    if p[i] == b'#' {
        panic!("Error while reading PGM: comments not allowed");
    }
    let w = read_uint(p, &mut i);
    advance_space(p, &mut i);
    let h = read_uint(p, &mut i);
    advance_space(p, &mut i);
    let max = read_uint(p, &mut i);
    ImgData {w, h, max, data: &p[i+1..]}
}

fn glyph_data(img: ImgData,
    cols: usize, rows: usize, w: usize, h: usize, shiftw: usize, shifth: usize
) -> Vec<u8> {
    let mut acc: Vec<u8> = Vec::new();
    let data = img.data;
    let imgw = img.w as usize;
    for row in 0..rows {
        let hstart = row*shifth;
        for col in 0..cols {
            let wstart = col*shiftw;
            for y in hstart..hstart+h {
                for x in wstart..wstart+w {
                    acc.push(data[y*imgw + x]);
                }
            }
        }
    }
    acc
}

pub fn pgm_as_glyph_data(pgm: &[u8],
    cols: usize, rows: usize, w: usize, h: usize, shiftw: usize, shifth: usize
) -> GlyphData {
    let img = pnm_as_data(pgm, Fmt::PGM);
    GlyphData {width: img.w, height: img.h,
        data: glyph_data(img, cols, rows, w, h, shiftw, shifth)}
}

pub fn pnm_as_single_image(pnm: &[u8], fmt: Fmt) -> GlyphData {
    let img = pnm_as_data(pnm, fmt);
    let w = img.w;
    let h = img.h;
    GlyphData{width: w, height: h, data: img.data.to_vec()}
}
