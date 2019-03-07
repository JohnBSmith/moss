
use std::time::{Duration,Instant};
use std::thread::sleep;
use std::rc::Rc;
use crate::math::type_error_int_float;
use crate::object::{Env,Object,Function,FnResult,new_module};

fn time_clock(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"clock")
    }
    let clock = Instant::now();
    let f = Box::new(move |_env: &mut Env, _pself: &Object, _argv: &[Object]|
    -> FnResult
    {
        let elapsed = clock.elapsed();
        let s = elapsed.as_secs() as f64;
        let us = elapsed.subsec_micros() as f64;
        return Ok(Object::Float(s+0.000001*us));
    });
    return Ok(Function::mutable(f,0,0));
}

fn time_sleep(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"sleep")
    }
    let duration = match argv[0] {
        Object::Float(x) => {
            let x = if x<0.0 {0.0} else {x};
            Duration::from_micros((1000000.0*x) as u64)
        },
        Object::Int(x) => {
            let x = if x<0 {0} else {x};
            Duration::from_secs(x as u64)
        },
        ref x => return type_error_int_float(env,"sleep",x)
    };
    sleep(duration);
    return Ok(Object::Null);
}

pub fn load_time() -> Object {
    let time = new_module("time");
    {
        let mut m = time.map.borrow_mut();
        m.insert_fn_plain("sleep",time_sleep,1,1);
        m.insert_fn_plain("clock",time_clock,0,0);
    }
    return Object::Interface(Rc::new(time));
}

