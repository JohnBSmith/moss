
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::rc::Rc;
use std::convert::TryFrom;
use crate::math::type_error_int_float;
use crate::object::{
    Env, Object, Function, FnResult,
    float, new_module
};

fn time_clock(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"clock")
    }
    let clock = Instant::now();
    let f = Box::new(move |_env: &mut Env, _pself: &Object, _argv: &[Object]|
    -> FnResult
    {
        let elapsed = clock.elapsed();
        let s = float(elapsed.as_secs());
        let us = float(elapsed.subsec_micros());
        Ok(Object::Float(s+0.000001*us))
    });
    Ok(Function::mutable(f,0,0))
}

fn time_sleep(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"sleep")
    }
    let duration = match argv[0] {
        Object::Float(x) => {
            Duration::from_micros((1000000.0*x).max(0.0) as u64)
        },
        Object::Int(x) => {
            Duration::from_secs(u64::try_from(x).unwrap_or(0))
        },
        ref x => return type_error_int_float(env, "sleep", x)
    };
    sleep(duration);
    Ok(Object::Null)
}

pub fn load_time() -> Object {
    let time = new_module("time");
    {
        let mut m = time.map.borrow_mut();
        m.insert_fn_plain("sleep", time_sleep, 1, 1);
        m.insert_fn_plain("clock", time_clock, 0, 0);
    }
    Object::Interface(Rc::new(time))
}

