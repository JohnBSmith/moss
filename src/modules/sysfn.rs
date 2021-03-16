
use std::rc::Rc;

use crate::object::{Object, FnResult, Function, PlainFn};
use crate::vm::{RTE,Env};
use crate::vm::{frame_info, frame_stack_len};

fn info(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    let index = match argv.len() {
        0 => match frame_stack_len(env) {
            0 => return Ok(Object::Null),
            n => n-1
        },
        1 => match argv[0] {
            Object::Int(value) => value as usize,
            _ => panic!()
        },
        n => return env.argc_error(n,0,1,"info")
    };
    let frame = frame_info(env,index);
    let id = Object::from(&*frame.file);
    let line = Object::Int(frame.line as i32);
    let col = Object::Int(frame.col as i32);
    let name = frame.name;
    Ok(Object::from(vec![id, line, col, name]))
}

fn stack_len(env: &mut Env, _pself: &Object, _argv: &[Object]) -> FnResult {
    Ok(Object::Int(frame_stack_len(env) as i32))
}

/*
== Backlinks ==
info, stack_len: sys.trace, sys.inspect
*/

static FN_TABLE: [(PlainFn, u32, u32); 2] = [
    (info, 0, 1),
    (stack_len, 0, 0)
];

fn sysfn(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"sysfn")
    }
    let index = match argv[0] {
        Object::Int(value) => value as usize,
        _ => return Ok(Object::Null)
    };
    let (f,min,max) = FN_TABLE[index];
    Ok(Function::plain(f,min,max))
}

pub fn load_sysfn(_rte: &Rc<RTE>) -> Object {
    Function::plain(sysfn,1,1)
}
