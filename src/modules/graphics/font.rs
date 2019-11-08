
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
    loop{
        let c = p[i];
        if c==b' ' || c==b'\n' || c==b'\t' || c==b'\r' {i+=1}
        else {break;}
    }
    *index = i;
}

fn read_uint(p: &[u8], j: &mut usize) -> u32 {
    let mut i = *j;
    let mut x: u32 = 0;
    loop{
        let c = p[i];
        if b'0'<=c && c<=b'9' {
            x = 10*x+(c as u32)-(b'0' as u32);
            i+=1;
        }
        else {break;}
    }
    *j = i;
    return x;
}

fn pgm_as_data(pgm: &[u8]) -> ImgData {
    if pgm[0]!=b'P' || pgm[1]!=b'5' {panic!("Error: not a PGM");}
    let mut i = 2;
    advance_space(pgm,&mut i);
    if pgm[i]==b'#' {
        panic!("Error while reading PGM: comments not allowed");
    }
    let w = read_uint(pgm,&mut i);
    advance_space(pgm,&mut i);
    let h = read_uint(pgm,&mut i);
    advance_space(pgm,&mut i);
    let max = read_uint(pgm,&mut i);
    return ImgData{w,h,max,data: &pgm[i+1..]};
}

fn glyph_data(img: ImgData,
    cols: usize, rows: usize, w: usize, h: usize, shiftw: usize, shifth: usize
) -> Vec<u8> {
    let mut buffer: Vec<u8> = Vec::new();
    let data = img.data;
    let imgw = img.w as usize;
    for row in 0..rows {
        let hstart = row*shifth;
        for col in 0..cols {
            let wstart = col*shiftw;
            for y in hstart..hstart+h {
                for x in wstart..wstart+w {
                    buffer.push(data[y*imgw+x]);
                }
            }
        }
    }
    return buffer;
}

pub fn pgm_as_glyph_data(pgm: &[u8],
    cols: usize, rows: usize, w: usize, h: usize, shiftw: usize, shifth: usize
) -> GlyphData {
    let img = pgm_as_data(pgm);
    return GlyphData{width: img.w, height: img.h,
        data: glyph_data(img,cols,rows,w,h,shiftw,shifth)
    };
}

pub fn pgm_as_single_image(pgm: &[u8]) -> GlyphData {
    let img = pgm_as_data(pgm);
    let w = img.w;
    let h = img.h;
    return GlyphData{width: w, height: h,
        data: glyph_data(img,1,1,w as usize,h as usize,0,0)
    };
}
