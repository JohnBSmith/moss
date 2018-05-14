
use object::{Object, FnResult};
use vm::Env;
use std::f64::NAN;

pub struct Long{}

impl Long {
    pub fn to_long(_x: &Object) -> Result<Object,()> {Err(())}
    pub fn downcast(_x: &Object) -> Option<&Long> {None}
    pub fn as_f64(&self) -> f64 {NAN}
    pub fn add_int_int(_a: i32, _b: i32) -> Object {
        panic!("Overflow in a+b.")
    }
    pub fn sub_int_int(a: i32, b: i32) -> Object {
        panic!("Overflow in a-b.")
    }
    pub fn mpy_int_int(_a: i32, _b: i32) -> Object {
        panic!("Overflow in a+b.")
    }
    pub fn pow_int_uint(_a: i32, _b: u32) -> Object {
        panic!("Overflow in a+b.")
    }
}

pub fn pow_mod(env: &mut Env, _a: &Object, _n: &Object, _m: &Object) -> FnResult {
    env.std_exception("Error: pow(x,n,m) is unimplemented.")
}
