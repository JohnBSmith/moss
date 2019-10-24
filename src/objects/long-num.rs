
use std::rc::Rc;
use std::any::Any;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::object::{
    Object, FnResult, Interface, Exception,
    downcast, ptr_eq_plain
};
use crate::vm::{Env, RTE};
use num_bigint::{BigInt,Sign};
use num_traits::{Zero,Pow};
use num_traits::cast::ToPrimitive;

fn bigint_as_f64(x: &BigInt) -> f64 {
    match x.to_f64() {
        Some(value) => value,
        None => std::f64::NAN
    }
}

pub struct Long{
    value: BigInt
}

impl Long {
    #[allow(dead_code)]
    pub fn from_int(x: i32) -> Long {
        Long{value: BigInt::from(x)}
    }

    pub fn object_from_int(x: i32) -> Object {
        Object::Interface(Rc::new(Long{value: BigInt::from(x)}))
    }
    
    pub fn object_from_string(a: &[char]) -> Result<Object,()> {
        let s: String = a.iter().collect();
        match BigInt::parse_bytes(s.as_bytes(),10) {
            Some(y) => {
                Ok(Object::Interface(Rc::new(Long{value: y})))
            },
            None => Err(())
        }
    }

    pub fn to_long(x: &Object) -> Result<Object,()> {
        return match *x {
            Object::Int(x) => {
                Ok(Long::object_from_int(x))
            },
            Object::String(ref s) => {
                Long::object_from_string(&s.data)
            },
            Object::Interface(ref x) => {
                if let Some(_) = x.as_any().downcast_ref::<Long>() {
                    Ok(Object::Interface(x.clone()))
                }else{
                    Err(())
                }
            },
            _ => Err(())
        }
    }

    pub fn as_f64(&self) -> f64 {
        bigint_as_f64(&self.value)
    }
    pub fn try_as_int(&self) -> Result<i32,()> {
        match self.value.to_i32() {
            Some(value) => Ok(value),
            None => Err(())
        }
    }
    pub fn add_int_int(a: i32, b: i32) -> Object {
        let x = BigInt::from(a);
        return Object::Interface(Rc::new(Long{value: x+b}));
    }
    pub fn sub_int_int(a: i32, b: i32) -> Object {
        let x = BigInt::from(a);
        return Object::Interface(Rc::new(Long{value: x-b}));
    }
    pub fn mul_int_int(a: i32, b: i32) -> Object {
        let x = BigInt::from(a);
        return Object::Interface(Rc::new(Long{value: x*b}));
    }
    pub fn pow_int_uint(a: i32, b: u32) -> Object {
        let x = BigInt::from(a);
        return Object::Interface(Rc::new(Long{value: x.pow(b)}));
    }
}

impl Interface for Long {
    fn as_any(&self) -> &dyn Any {self}
    fn type_name(&self, _env: &mut Env) -> String {
        "Long".to_string()
    }
    fn get_type(&self, env: &mut Env) -> FnResult {
        Ok(Object::Interface(env.rte().type_long.clone()))
    }
    fn to_string(self: Rc<Self>, _env: &mut Env) -> Result<String,Box<Exception>> {
        Ok(self.value.to_string())
    }

    fn add(self: Rc<Self>, b: &Object, _env: &mut Env) -> FnResult {
        if let Object::Int(b) = *b {
            let value = self.value.clone()+b;
            return Ok(Object::Interface(Rc::new(Long{value})));
        }else if let Some(b) = downcast::<Long>(b) {
            let value = self.value.clone()+b.value.clone();
            return Ok(Object::Interface(Rc::new(Long{value})));
        }else if let Object::Float(b) = *b {
            let a = bigint_as_f64(&self.value);
            return Ok(Object::Float(a+b));
        }else{
            return Ok(Object::unimplemented());
        }
    }

    fn sub(self: Rc<Self>, b: &Object, _env: &mut Env) -> FnResult {
        if let Object::Int(b) = *b {
            let value = self.value.clone()-b;
            return Ok(Object::Interface(Rc::new(Long{value})));
        }else if let Some(b) = downcast::<Long>(b) {
            let value = self.value.clone()-b.value.clone();
            return Ok(Object::Interface(Rc::new(Long{value})));
        }else if let Object::Float(b) = *b {
            let a = bigint_as_f64(&self.value);
            return Ok(Object::Float(a-b));
        }else{
            return Ok(Object::unimplemented());
        }
    }

    fn mul(self: Rc<Self>, b: &Object, _env: &mut Env) -> FnResult {
        if let Object::Int(b) = *b {
            let value = self.value.clone()*b;
            return Ok(Object::Interface(Rc::new(Long{value})));
        }else if let Some(b) = downcast::<Long>(b) {
            let value = self.value.clone()*b.value.clone();
            return Ok(Object::Interface(Rc::new(Long{value})));
        }else if let Object::Float(b) = *b {
            let a = bigint_as_f64(&self.value);
            return Ok(Object::Float(a*b));
        }else{
            return Ok(Object::unimplemented());
        }
    }

    fn radd(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(a) = *a {
            let value = self.value.clone()+a;
            return Ok(Object::Interface(Rc::new(Long{value})));
        }else if let Object::Float(a) = *a {
            let b = bigint_as_f64(&self.value);
            return Ok(Object::Float(a+b));
        }else{
            return env.type_error("Type error in a+b: cannot add a and b: Long.");
        }
    }

    fn rsub(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(a) = *a {
            let value = a-self.value.clone();
            return Ok(Object::Interface(Rc::new(Long{value})));
        }else if let Object::Float(a) = *a {
            let b = bigint_as_f64(&self.value);
            return Ok(Object::Float(a-b));
        }else{
            return env.type_error("Type error in a-b: cannot subtract a and b: Long.");
        }
    }

    fn rmul(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(a) = *a {
            let value = self.value.clone()*a;
            return Ok(Object::Interface(Rc::new(Long{value})));
        }else if let Object::Float(a) = *a {
            let b = bigint_as_f64(&self.value);
            return Ok(Object::Float(a*b));
        }else{
            return env.type_error("Type error in x*y: cannot multiply x and y: Long.");
        }
    }

    fn div(self: Rc<Self>, b: &Object, _env: &mut Env) -> FnResult {
        let a = bigint_as_f64(&self.value);
        match *b {
            Object::Int(b) => return Ok(Object::Float(a/(b as f64))),
            Object::Float(b) => return Ok(Object::Float(a/b)),
            _ => {}
        }
        if let Some(b) = downcast::<Long>(b) {
            let b = bigint_as_f64(&b.value);
            return Ok(Object::Float(a/b));
        }
        Ok(Object::unimplemented())
    }
    
    fn rdiv(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        let b = bigint_as_f64(&self.value);
        return match *a {
            Object::Int(a) => Ok(Object::Float((a as f64)/b)),
            Object::Float(a) => Ok(Object::Float(a/b)),
            ref x => env.type_error1("Type error in x/y: cannot divide x by y: Long.","x",x)
        };
    }

    fn idiv(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(b) = *b {
            if b==0 {
                return env.value_error("Value error in a//b: b==0.");
            }
            // Todo: ensure floor division
            let value = self.value.clone()/b;
            return Ok(Object::Interface(Rc::new(Long{value})));
        }else if let Some(b) = downcast::<Long>(b) {
            if b.value==Zero::zero() {
                return env.value_error("Value error in a//b: b==0.");
            }
            // Todo: ensure floor division
            let value = self.value.clone()/b.value.clone();
            return Ok(Object::Interface(Rc::new(Long{value})));
        }else{
            return env.type_error("Type error in a//b.");
        }
    }

    fn ridiv(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(a) = *a {
            if self.value==Zero::zero() {
                return env.value_error("Value error in a//b: b==0.");
            }
            // Todo: ensure floor division
            let value = a/self.value.clone();
            return Ok(Object::Interface(Rc::new(Long{value})));
        }else{
            return env.type_error("Type error in a//b.");
        }
    }
    
    fn imod(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(b) = *b {
            // Todo: ensure floor division
            let value = self.value.clone()%b;
            return Ok(Object::Interface(Rc::new(Long{value})));
        }else if let Some(b) = downcast::<Long>(b) {
            // Todo: ensure floor division
            let value = self.value.clone()%b.value.clone();
            return Ok(Object::Interface(Rc::new(Long{value})));
        }else{
            return env.type_error("Type error in a%b: a: Long and b.");
        }
    }
    
    fn rimod(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(a) = *a {
            // Todo: ensure floor division
            let value = a%self.value.clone();
            return Ok(Object::Interface(Rc::new(Long{value})));
        }else{
            return env.type_error("Type error in a%b: a: Long and b.");
        }
    }

    fn pow(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(b) = *b {
            if b<0 {
                return env.value_error("Value error in a^b: b<0.");
            }
            let value = self.value.pow(b as u32);
            return Ok(Object::Interface(Rc::new(Long{value})));      
        }else{
            return env.type_error("Type error in a^b.");
        }
    }

    fn eq_plain(&self, b: &Object) -> bool {
        if let Object::Int(b) = *b {
            return self.value == BigInt::from(b);
        }else if let Some(b) = downcast::<Long>(b) {
            return self.value == b.value;
        }else{
            return false;
        }
    }

    fn req_plain(&self, a: &Object) -> bool {
        if let Object::Int(a) = *a {
            return self.value == BigInt::from(a);
        }else if let Some(a) = downcast::<Long>(a) {
            return self.value == a.value;
        }else{
            return false;
        }  
    }
    
    fn eq(self: Rc<Self>, b: &Object, _env: &mut Env) -> FnResult {
        return Ok(Object::Bool(self.eq_plain(b)));
    }

    fn req(self: Rc<Self>, a: &Object, _env: &mut Env) -> FnResult {
        return Ok(Object::Bool(self.req_plain(a)));
    }

    fn lt(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(b) = *b {
            return Ok(Object::Bool(self.value<BigInt::from(b)));
        }else if let Some(b) = downcast::<Long>(b) {
            return Ok(Object::Bool(self.value<b.value));
        }else{
            return env.type_error("Type error in a<b.");
        }
    }

    fn le(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(b) = *b {
            return Ok(Object::Bool(self.value<=BigInt::from(b)));
        }else if let Some(b) = downcast::<Long>(b) {
            return Ok(Object::Bool(self.value<=b.value));
        }else{
            return env.type_error("Type error in a<=b.");
        }
    }

    fn rlt(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(a) = *a {
            return Ok(Object::Bool(BigInt::from(a)<self.value));
        }else if let Some(a) = downcast::<Long>(a) {
            return Ok(Object::Bool(a.value<self.value));
        }else{
            return env.type_error("Type error in a<b.");
        }
    }

    fn rle(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(a) = *a {
            return Ok(Object::Bool(BigInt::from(a)<=self.value));
        }else if let Some(a) = downcast::<Long>(a) {
            return Ok(Object::Bool(a.value<=self.value));
        }else{
            return env.type_error("Type error in a<b.");
        }
    }

    fn abs(self: Rc<Self>, _env: &mut Env) -> FnResult {
        let value = if self.value<Zero::zero() {
            -self.value.clone()
        }else{
            self.value.clone()
        };
        return Ok(Object::Interface(Rc::new(Long{value})));
    }

    fn sgn(self: Rc<Self>, _env: &mut Env) -> FnResult {
        let sign: i32 = match self.value.sign() {
            Sign::Plus => 1,
            Sign::Minus => -1,
            Sign::NoSign => 0
        };
        return Ok(Object::Int(sign));
    }

    fn neg(self: Rc<Self>, _env: &mut Env) -> FnResult {
        let value = -self.value.clone();
        return Ok(Object::Interface(Rc::new(Long{value})));
    }

    fn is_instance_of(&self, type_obj: &Object, rte: &RTE) -> bool {
        if let Object::Interface(p) = type_obj {
            ptr_eq_plain(p,&rte.type_long)
        }else{false}
    }

    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.value.hash(&mut hasher);
        return hasher.finish();
    }
}

fn to_bigint(x: &Object) -> Result<BigInt,()> {
    if let Object::Int(x) = *x {
        return Ok(BigInt::from(x));
    }else if let Some(x) = downcast::<Long>(x) {
        return Ok(x.value.clone());
    }else{
        return Err(());
    }
}

pub fn pow_mod(env: &mut Env, a: &Object, n: &Object, m: &Object) -> FnResult {
    let a = match to_bigint(a) {
        Ok(x) => x,
        Err(()) => return env.type_error("Type error in pow(a,n,m): expected a of type Int or Long.")
    };
    let n = match to_bigint(n) {
        Ok(x) => x,
        Err(()) => return env.type_error("Type error in pow(a,n,m): expected n of type Int or Long.")
    };
    let m = match to_bigint(m) {
        Ok(x) => x,
        Err(()) => return env.type_error("Type error in pow(a,n,m): expected m of type Int or Long.")
    };
    if n<Zero::zero() {
        return env.value_error("Value error in pow(a,n,m): n<0.");
    }
    return Ok(Object::Interface(Rc::new(Long{value: a.modpow(&n,&m)})));
}

