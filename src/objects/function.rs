
#![allow(unused_variables)]
#![allow(unused_imports)]

use std::rc::Rc;
use std::cell::RefCell;
use std::i32;

use object::{
    Object, FnResult, U32String, Function, Table, List,
    VARIADIC, MutableFn, EnumFunction, Range
};
use vm::Env;
use iterable::new_iterator;

fn apply(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    if argv.len()==1 {
        match argv[0] {
            Object::List(ref a) => {
                env.call(pself,&Object::Null,&a.borrow().v)
            },
            ref a => env.type_error1(
                "Type error in f.apply(a): a is not a list.","a",a)
        }
    }else if argv.len()==2 {
        match argv[1] {
            Object::List(ref a) => {
                env.call(pself,&argv[0],&a.borrow().v)
            },
            ref a => env.type_error1(
                "Type error in f.apply(a): a is not a list.","a",a)
        }
    }else{
        return env.argc_error(argv.len(),1,1,"apply");
    }
}

fn orbit(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    if argv.len()!=1 {
        return env.argc_error(argv.len(),1,1,"orbit");
    }
    let mut x = argv[0].clone();
    let f = pself.clone();
    let i = Box::new(
        move |env: &mut Env, pself: &Object, argv: &[Object]| -> FnResult {
            let y = x.clone();
            x = try!(env.call(&f,&Object::Null,&[x.clone()]));
            return Ok(y);
        }
    );
    return Ok(new_iterator(i));
}

fn argc(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    if argv.len()!=0 {
        return env.argc_error(argv.len(),0,0,"argc");
    }
    if let Object::Function(ref f) = *pself {
        if f.argc == VARIADIC {
            let min = Object::Int(f.argc_min as i32);
            let max = if f.argc_max == VARIADIC {
                Object::Null
            }else{
                Object::Int(f.argc_max as i32)
            }; 
            Ok(Object::Range(Rc::new(Range{
                a: min, b: max, step: Object::Null
            })))
        }else if f.argc > i32::MAX as u32 {
            env.value_error("Value error f.argc(): the count is too large to be represented as i32.")
        }else{
            Ok(Object::Int(f.argc as i32))
        }
    }else{
        env.type_error1("Type error in f.argc(): f is not a function.","f",pself)
    }
}

pub fn iterate(env: &mut Env, f: &Object, n: &Object) -> FnResult {
    match *n {
        Object::Int(n) => {
            let f = f.clone();
            let g = move |env: &mut Env, pself: &Object, argv: &[Object]| -> FnResult {
                let mut y = argv[0].clone();
                for _ in 0..n {
                    y = try!(env.call(&f,&Object::Null,&[y]));
                }
                return Ok(y);
            };
            Ok(Function::mutable(Box::new(g),1,1))
        },
        ref n => env.type_error1("Type error in f^n: n is not an integer.","n",n)
    }
}

pub fn init(t: &Table){
    let mut m = t.map.borrow_mut();
    m.insert_fn_plain("apply",apply,1,1);
    m.insert_fn_plain("orbit",orbit,1,1);
    m.insert_fn_plain("argc", argc,0,0);
}
