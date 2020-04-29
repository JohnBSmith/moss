
use std::rc::Rc;

use crate::object::{Object, FnResult, Function, PlainFn};
use crate::vm::{RTE,Env};

static FN_TABLE: [(PlainFn,u32,u32); 0] = [];

fn sysfn(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"sysfn")
    }
    let index = match argv[0] {
        Object::Int(value) => value as usize,
        _ => return Ok(Object::Null)
    };
    let (f,min,max) = FN_TABLE[index];
    return Ok(Function::plain(f,min,max));
}

pub fn load_sysfn(_rte: &Rc<RTE>) -> Object {
    return Function::plain(sysfn,1,1);
}
