
use std::rc::Rc;
use crate::object::{Env, Object, FnResult, new_module};
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
    let a = vec![id,Object::Int(frame.line as i32)];
    return Ok(Object::from(a));
}

fn stack_len(env: &mut Env, _pself: &Object, _argv: &[Object]) -> FnResult {
    return Ok(Object::Int(frame_stack_len(env) as i32));
}

pub fn load_inspect() -> Object {
    let inspect = new_module("inspect");
    {
        let mut m = inspect.map.borrow_mut();
        m.insert_fn_plain("info",info,0,1);
        m.insert_fn_plain("stack_len",stack_len,0,0);
    }
    return Object::Interface(Rc::new(inspect));
}
