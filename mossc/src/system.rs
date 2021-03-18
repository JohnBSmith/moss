
use std::fs::File;
use std::io::Write;

use generator::CodeObject;

fn push_u32(bv: &mut Vec<u8>, x: u32) {
    bv.push(x as u8);
    bv.push((x>>8) as u8);
    bv.push((x>>16) as u8);
    bv.push((x>>24) as u8);
}

fn write_u32(bv: &mut [u8], x: u32) {
    bv[0] = x as u8;
    bv[1] = (x>>8) as u8;
    bv[2] = (x>>16) as u8;
    bv[3] = (x>>24) as u8;
}

fn push_string(bv: &mut Vec<u8>, s: &str) {
    push_u32(bv, s.len() as u32);
    for c in s.as_bytes() {
        bv.push(*c);
    }
}

fn serialize(code: &CodeObject) -> Vec<u8> {
    let mut bv: Vec<u8> = Vec::with_capacity(code.program.len());
    push_u32(&mut bv, 0); // indicates a binary file
    push_u32(&mut bv, 0xcafe); // pointer to data
    for x in &code.program {
        push_u32(&mut bv, *x);
    }
    let len = bv.len() as u32;
    write_u32(&mut bv[4..8], len);

    let len = code.data.len() as u32;
    push_u32(&mut bv, len);
    for x in &code.data {
        push_string(&mut bv, x);
    }
    bv
}

pub fn save(code: &CodeObject, id: &str) {
    let bv = serialize(code);
    let mut file = File::create(format!("{}.bin", id)).unwrap();
    file.write_all(&bv).unwrap();
}

