
use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;
use std::io::{Read,Write};

use system::History;
use object::{Object,Exception,Map};
use vm::{Module,RTE,Env,eval};
use compiler::EnumError;

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

pub fn push_object(bv: &mut Vec<u8>, x: &Object) {
    if let Object::String(s) = x {
        let s = s.to_string();
        push_u32(bv,s.len() as u32);
        for c in s.as_bytes() {
            bv.push(*c);
        }
    }else{
        unimplemented!();
    }
}

fn serialize(v: &[u32], data: &[Object]) -> Vec<u8> {
    let mut bv: Vec<u8> = Vec::with_capacity(v.len());
    push_u32(&mut bv,0); // indicates a binary file
    push_u32(&mut bv,0xcafe); // pointer to data
    for x in v {
        push_u32(&mut bv,*x);
    }
    let len = bv.len() as u32;
    write_u32(&mut bv[4..8],len);

    let len = data.len() as u32;
    push_u32(&mut bv, len);
    for x in data {
        push_object(&mut bv,x);
    }
    return bv;
}

fn load_u32(bv: &[u8]) -> u32 {
       bv[0] as u32
    | (bv[1] as u32)<<8
    | (bv[2] as u32)<<16
    | (bv[3] as u32)<<24
}

fn load_object(bv: &[u8], index: usize) -> (usize,Object) {
    let i = index+4;
    let size = load_u32(&bv[index..i]) as usize;
    let a: Vec<u8> = Vec::from(&bv[i..i+size]);
    let s = match String::from_utf8(a) {
        Ok(value) => value,
        Err(_) => panic!("Could not load binary module: invalid Unicode.")
    };
    return (i+size,Object::from(&s[..]));
}

fn load_from_u8(rte: &Rc<RTE>, id: &str, bv: &[u8])
-> Result<Rc<Module>,Box<EnumError>>
{
    let data_index = load_u32(&bv[4..8]) as usize;
    let program_size = (data_index-8)/4;
    let mut v: Vec<u32> = Vec::with_capacity(program_size);
    let mut i = 8;
    while i<data_index {
        v.push(load_u32(&bv[i..i+4]));
        i+=4;
    }
    let data_count = load_u32(&bv[data_index..data_index+4]) as usize;
    let mut data: Vec<Object> = Vec::with_capacity(data_count);
    i = data_index+4;
    for _ in 0..data_count {
        let (index,x) = load_object(bv,i);
        data.push(x);
        i = index;
    }
    let m = Rc::new(Module{
        program: Rc::from(v),
        data: data,
        rte: rte.clone(),
        gtab: rte.gtab.clone(),
        id: id.to_string()
    });
    return Ok(m);
}

fn save_module(m: &Rc<Module>) {
    let bv = serialize(&m.program, &m.data);
    let mut file = File::create(format!("{}.bin",m.id)).unwrap();
    file.write_all(&bv).unwrap();
}

pub fn compile_file(rte: &Rc<RTE>, id: &str) {
    let mut f = match ::module::open_file(id) {
        Some(f) => f, None => return
    };
    let mut s = String::new();
    f.read_to_string(&mut s).expect("something went wrong reading the file");

    let history = &mut History::new();
    match ::compiler::scan(&s,1,id,false) {
        Ok(v) => {
            match ::compiler::compile(v,false,::compiler::Value::Optional,history,id,rte) {
                Ok(module) => save_module(&module),
                Err(e) => ::compiler::print_error(&e)
            };
        },
        Err(error) => ::compiler::print_error(&error)
    }
}

fn load_module(rte: &Rc<RTE>, f: &mut File, id: &str)
-> Result<Rc<Module>,Box<EnumError>>
{
    let mut bv: Vec<u8> = Vec::new();
    match f.read_to_end(&mut bv) {
        Ok(_) => {},
        Err(_) => panic!()
    }
    return load_from_u8(rte,id,&bv);
}

pub fn eval_module(env: &mut Env, gtab: Rc<RefCell<Map>>, f: &mut File, id: &str)
-> Result<Object,Box<Exception>>
{
    let m = match load_module(env.rte(),f,id) {
        Ok(m) => m,
        Err(_) => panic!()
    };
    return eval(env,m,gtab,false);
}

pub fn open_file(id: &str) -> Option<File> {
    let mut path: String = String::from(id);
    path += ".moss";
    return match File::open(&path) {
        Ok(f) => Some(f),
        Err(_) => {
            match File::open(id) {
                Ok(f) => Some(f),
                Err(_) => {
                    println!("File '{}' not found.",id);
                    return None;
                }
            }
        }
    };
}

