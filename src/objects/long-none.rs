
use object::{Object, FnResult, OperatorResult};
use vm::{Env,EnvPart};
use std::f64::NAN;

pub struct Long{}

impl Long {
    pub fn to_long(_x: &Object) -> Result<Object,()> {Err(())}
    pub fn as_f64(&self) -> f64 {NAN}
    pub fn object_from_string(_a: &[char]) -> Result<Object,()> {Err(())}
    pub fn try_as_int(&self) -> Result<i32,()> {Err(())}
}

pub fn pow_mod(env: &mut Env, _a: &Object, _n: &Object, _m: &Object) -> FnResult {
    env.std_exception("Error: pow(x,n,m) is unimplemented.")
}

#[inline(never)]
fn overflow_exception(env: &EnvPart, op: &str, x: i32, y: i32)
-> OperatorResult
{
    Err(env.std_exception_plain(&format!(
        "Integer overflow in x{}y. Note: x={}, y={}.",op,x,y
    )))
}

pub fn overflow_from_add(env: &EnvPart, x: i32, y: i32) -> OperatorResult {
    overflow_exception(env,"+",x,y)
}

pub fn overflow_from_sub(env: &EnvPart, x: i32, y: i32) -> OperatorResult {
    overflow_exception(env,"-",x,y)
}

pub fn overflow_from_mul(env: &EnvPart, x: i32, y: i32) -> OperatorResult {
    overflow_exception(env,"*",x,y)
}

pub fn overflow_from_idiv(env: &EnvPart, x: i32, y: i32) -> OperatorResult {
    overflow_exception(env,"//",x,y)
}

pub fn overflow_from_pow(env: &EnvPart, x: i32, y: i32) -> OperatorResult {
    overflow_exception(env,"^",x,y)
}
