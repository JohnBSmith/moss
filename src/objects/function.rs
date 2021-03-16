
use std::rc::Rc;

use crate::object::{
    Object, FnResult, Function, VARIADIC
};
use crate::vm::Env;
use crate::iterable::new_iterator;
use crate::range::Range;
use crate::class::Class;

fn orbit(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"orbit")
    }
    let mut x = argv[0].clone();
    let f = pself.clone();
    let i = Box::new(
        move |env: &mut Env, _pself: &Object, _argv: &[Object]| -> FnResult {
            let y = x.clone();
            x = env.call(&f, &Object::Null, &[x.clone()])?;
            Ok(y)
        }
    );
    Ok(new_iterator(i))
}

fn argc(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"argc")
    }
    if let Object::Function(ref f) = *pself {
        if f.argc == VARIADIC {
            let min = Object::Int(f.argc_min as i32);
            let max = if f.argc_max == VARIADIC {
                Object::Null
            } else {
                Object::Int(f.argc_max as i32)
            }; 
            Ok(Object::Interface(Rc::new(Range {
                a: min, b: max, step: Object::Null
            })))
        } else if f.argc > i32::MAX as u32 {
            env.value_error("Value error f.argc(): the count is too large to be represented as i32.")
        } else {
            Ok(Object::Int(f.argc as i32))
        }
    } else {
        env.type_error1("Type error in f.argc(): f is not a function.", "f", pself)
    }
}

pub fn iterate(env: &mut Env, f: &Object, n: &Object) -> FnResult {
    match *n {
        Object::Int(n) => {
            let f = f.clone();
            let g = move |env: &mut Env, _pself: &Object, argv: &[Object]| -> FnResult {
                let mut y = argv[0].clone();
                for _ in 0..n {
                    y = env.call(&f, &Object::Null, &[y])?;
                }
                Ok(y)
            };
            Ok(Function::mutable(Box::new(g),1,1))
        },
        ref n => env.type_error1("Type error in f^n: n is not an integer.", "n", n)
    }
}

pub fn init(t: &Class){
    let mut m = t.map.borrow_mut();
    m.insert_fn_plain("orbit", orbit, 1, 1);
    m.insert_fn_plain("argc", argc, 0, 0);
}
