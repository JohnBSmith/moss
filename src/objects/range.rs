
use std::rc::Rc;
use std::any::Any;
use std::char;
use object::{
    Object, Env, Interface, FnResult, Exception,
    CharString, downcast
};
use vm::object_to_string;
use iterable::{new_iterator, int_range_iterator};

pub struct Range {
    pub a: Object,
    pub b: Object,
    pub step: Object
}

impl Interface for Range {
    fn as_any(&self) -> &Any {self}
    fn type_name(&self, _env: &mut Env) -> String {String::from("Range")}

    fn to_string(self: Rc<Self>, env: &mut Env) -> Result<String,Box<Exception>> {
        let a = object_to_string(env,&self.a)?;
        let b = object_to_string(env,&self.b)?;
        Ok(match self.step {
            Object::Null =>
                format!("{}..{}",a,b),
            ref step =>
                format!("{}..{}: {}",a,b,object_to_string(env,step)?)
        })
    }

    fn eq_plain(&self, b: &Object) -> bool {
        if let Some(y) = downcast::<Range>(b) {
            self.a==y.a && self.b==y.b && self.step==y.step
        }else{
            false
        }
    }

    fn eq(self: Rc<Self>, b: &Object, _env: &mut Env) -> FnResult {
        Ok(if let Some(y) = downcast::<Range>(b) {
            Object::Bool(self.a==y.a && self.b==y.b && self.step==y.step)
        }else{
            Object::unimplemented()
        })
    }
    
    fn get(&self, key: &Object, env: &mut Env) -> FnResult {
        match env.rte().type_iterable.map.borrow().m.get(key) {
            Some(x) => Ok(x.clone()),
            None => env.index_error(&format!(
                "Index error in r.{0}: '{0}' not in property chain.",
                key
            ))
        }
    }

    fn rin(&self, key: &Object, env: &mut Env) -> FnResult {
        let k = match *key {
            Object::Int(x) => x,
            _ => return Err(env.type_error_plain("Type error in 'k in i..j': k is not an integer."))
        };
        let i = match self.a {
            Object::Int(x) => x,
            _ => return Err(env.type_error_plain("Type error in 'k in i..j': i is not an integer."))
        };
        let j = match self.b {
            Object::Int(x) => x,
            _ => return Err(env.type_error_plain("Type error in 'k in i..j': j is not an integer."))
        };
        match self.step {
            Object::Null => {},
            _ => return Err(env.type_error_plain("Type error in 'k in i..j: step': step is not supported."))
        }
        return Ok(Object::Bool(i<=k && k<=j));
    }

    fn iter(self: Rc<Self>, env: &mut Env) -> FnResult {
        let mut a = match self.a {
            Object::Int(a)=>a,
            Object::Float(_) => return float_range_iterator(env,&self),
            Object::String(_) => return char_range_iterator(env,&self),
            _ => {return env.type_error("Type error in iter(a..b): a is not an integer.");}
        };
        let d = match self.step {
            Object::Null => 1,
            Object::Float(_) => return float_range_iterator(env,&self),
            Object::Int(x)=>x,
            _ => return env.type_error1(
                "Type error in iter(a..b: d): d is not an integer.",
                "d",&self.step)
        };
        if d==0 {
            return env.value_error("Value error in iter(a..b: d): d==0.");
        }
        let f: Box<FnMut(&mut Env,&Object,&[Object])->FnResult> = match self.b {
            Object::Int(b) => {
                if d<0 {
                    Box::new(move |_env: &mut Env, _pself: &Object, _argv: &[Object]| -> FnResult{
                        return if a>=b {
                            a+=d;
                            Ok(Object::Int(a-d))
                        }else{
                            Ok(Object::empty())
                        }
                    })
                }else{
                    int_range_iterator(a,b,d)
                }
            },
            Object::Null => {
                Box::new(move |_env: &mut Env, _pself: &Object, _argv: &[Object]| -> FnResult{
                    a+=d; Ok(Object::Int(a-d))
                })
            },
            _ => {return env.type_error("Type error in iter(a..b): b is not an integer.");}
        };
        return Ok(new_iterator(f));

    }
}

fn float_range_iterator(env: &mut Env, r: &Range) -> FnResult {
    let a = match r.a {
        Object::Int(x) => x as f64,
        Object::Float(x) => x,
        _ => return env.type_error1(
            "Type error in iter(a..b): a is not of type Float.",
            "a",&r.a)
    };
    let b = match r.b {
        Object::Int(x) => x as f64,
        Object::Float(x) => x,
        _ => return env.type_error1(
            "Type error in iter(a..b): b is not of type Float.",
            "b",&r.b)
    };
    let d = match r.step {
        Object::Null => 1.0,
        Object::Int(x) => x as f64,
        Object::Float(x) => x,
        _ => return env.type_error1(
            "Type error in iter(a..b: d): d is not of type Float.",
            "d",&r.step)
    };

    let q = (b-a)/d;
    let n = if q<0.0 {0} else {(q+0.001) as usize+1};
    let mut k = 0;

    let f = Box::new(move |_env: &mut Env, _pself: &Object, _argv: &[Object]| -> FnResult {
        return Ok(if k<n {
            let y = a+k as f64*d;
            k+=1;
            Object::Float(y)
        }else{
            Object::empty()
        });
    });
    return Ok(new_iterator(f));
}

fn char_range_iterator(env: &mut Env, r: &Range) -> FnResult {
    let mut a = if let Object::String(ref s) = r.a {
        if s.data.len()==1 {s.data[0] as u32} else {
            return env.value_error("
            Value error in iter(a..b): a is not a string of size 1.")
        }
    }else{
        unreachable!()
    };
    let b = if let Object::String(ref s) = r.b {
        if s.data.len()==1 {s.data[0] as u32} else {
            return env.value_error(
            "Value error in iter(a..b): b is not a string of size 1.")
        }
    }else{
        return env.type_error(
        "Type error in iter(a..b): b is not of type String.")
    };
    let f = Box::new(move |_env: &mut Env, _pself: &Object, _argv: &[Object]| -> FnResult {
        return Ok(if a<=b {
            let value = match char::from_u32(a) {
                Some(c) => CharString::new_object_char(c),
                None=> Object::Null
            };
            a+=1;
            value
        }else{
            Object::empty()
        });
    });
    return Ok(new_iterator(f));
}
