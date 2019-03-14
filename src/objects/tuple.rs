
use std::rc::Rc;
use std::any::Any;

use crate::object::{Object, Exception, Interface, FnResult, downcast};
use crate::vm::{Env, op_eq};

pub struct Tuple{
    pub v: Vec<Object>
}

impl Tuple {
    pub fn new_object(v: Vec<Object>) -> Object {
        return Object::Interface(Rc::new(Tuple{v}));
    }
}

impl Interface for Tuple {
    fn as_any(&self) -> &dyn Any {self}
    fn type_name(&self, _env: &mut Env) -> String {
        "Tuple".to_string()
    }
    fn to_string(self: Rc<Self>, env: &mut Env) -> Result<String,Box<Exception>> {
        let mut s = String::from("(");
        let mut first = true;
        for x in &self.v {
            if first {first = false;} else {s.push_str(", ");}
            s.push_str(&x.string(env)?);
        }
        s.push_str(")");
        return Ok(s);
    }
    fn index(self: Rc<Self>, indices: &[Object], env: &mut Env) -> FnResult {
        match indices.len() {
            1 => {}, n => return env.argc_error(n,1,1,"tuple indexing")
        }
        let i = match indices[0] {
            Object::Int(i) => if i<0 {0} else {i as usize},
            ref i => return env.type_error1(
                "Type error in t[i]: i is not an integer.",
                "i", i
            )
        };
        if i < self.v.len() {
            Ok(self.v[i].clone())
        }else{
            env.index_error("Index error in t[i]: i is out of upper bound.")
        }
    }
    fn eq_plain(&self, b: &Object) -> bool {
        if let Some(b) = downcast::<Tuple>(b) {
            if self.v.len()==b.v.len() {
                for i in 0..self.v.len() {
                    if self.v[i] != b.v[i] {return false;}
                }
                return true;
            }else{
                return false;
            }
        }else{
            return false;
        }
    }
    fn eq(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        if let Some(b) = downcast::<Tuple>(b) {
            let len = self.v.len();
            if len == b.v.len() {
                for i in 0..len {
                    let y = op_eq(env,&self.v[i],&b.v[i])?;
                    if let Object::Bool(y) = y {
                        if !y {return Ok(Object::Bool(false));}
                    }else{
                        return env.type_error(
                            "Type error in t1==t2: t[i]==t[i] is not a boolean."
                        );
                    }
                }
                return Ok(Object::Bool(true));
            }else{
                return Ok(Object::Bool(false));
            }
        }else{
            return Ok(Object::Bool(false));
        }
    }
}
