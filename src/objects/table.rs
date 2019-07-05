
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;

const TYPE: usize = 0;
const NAME: usize = 1;
const PROTOTYPE: usize = 2;

use crate::object::{
    Object, List, Map, Interface, downcast, ptr_eq_plain,
    Exception, FnResult
};
use crate::vm::{Env, RTE, map_to_string};
use crate::tuple::Tuple;

pub struct Table {
    pub prototype: Object,
    pub map: Rc<RefCell<Map>>
}

impl Table {
    pub fn new(prototype: Object) -> Rc<Table> {
        Rc::new(Table{prototype: prototype, map: Map::new()})
    }

    pub fn slot(&self, key: &Object) -> Option<Object> {
        let mut p = self;
        loop{
            match p.map.borrow_mut().m.get(key) {
                Some(value) => {return Some(value.clone());},
                None => {
                    if let Some(t) = downcast::<Table>(&p.prototype) {
                        p = t;
                    }else{
                        return None;
                    }
                }
            }
        }
    }
}

impl Interface for Table {
    fn as_any(&self) -> &dyn Any {self}
    
    fn type_name(&self, env: &mut Env) -> String {
        let type_object = match self.get_type(env) {
            Ok(value) => value,
            Err(_) => panic!()
        };
        if let Object::Null = type_object {
            return "Table object".to_string();
        }else{
            match type_object.string(env) {
                Ok(value) => return value,
                Err(_) => panic!()
            }
        }
    }

    fn to_string(self: Rc<Self>, env: &mut Env) -> Result<String,Box<Exception>> {
        if let Some(f) = type_get(&self.prototype,&env.rte().key_string) {
            let s = env.call(&f,&Object::Interface(self.clone()),&[])?;
            return s.string(env);
        }
        let left = if let Object::Null = self.prototype {
            "table{"
        }else{
            "table(...){"
        };
        match self.map.try_borrow_mut() {
            Ok(m) => map_to_string(env,&m.m,left,"}"),
            Err(_) => Ok(format!("{}{}",left,"...}"))
        }
    }

    fn eq_plain(&self, b: &Object) -> bool {
        return if let Some(y) = downcast::<Table>(b) {
            return self as *const Table == y as *const Table;
        }else{
            false
        }
    }

    fn neg(self: Rc<Self>, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_neg) {
            Some(ref f) => env.call(f,&Object::Interface(self),&[]),
            None => env.type_error1("Type error in -x.", "x",
                &Object::Interface(self))
        }
    }

    fn add(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_add) {
            Some(ref f) => {
                let x = Object::Interface(self);
                env.call(f,&x,&[y.clone()])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }
    
    fn radd(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_radd) {
            Some(ref f) => {
                env.call(f,&x,&[Object::Interface(self)])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }

    fn sub(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_sub) {
            Some(ref f) => {
                let x = Object::Interface(self);
                env.call(f,&x,&[y.clone()])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }

    fn rsub(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_rsub) {
            Some(ref f) => {
                env.call(f,&x,&[Object::Interface(self)])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }

    fn mul(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_mul) {
            Some(ref f) => {
                let x = Object::Interface(self);
                env.call(f,&x,&[y.clone()])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }

    fn rmul(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_rmul) {
            Some(ref f) => {
                env.call(f,&x,&[Object::Interface(self)])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }

    fn div(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_div) {
            Some(ref f) => {
                let x = Object::Interface(self);
                env.call(f,&x,&[y.clone()])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }

    fn rdiv(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_rdiv) {
            Some(ref f) => {
                env.call(f,&x,&[Object::Interface(self)])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }

    fn idiv(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_idiv) {
            Some(ref f) => {
                let x = Object::Interface(self);
                env.call(f,&x,&[y.clone()])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }

    fn ridiv(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_ridiv) {
            Some(ref f) => {
                env.call(f,&x,&[Object::Interface(self)])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }

    fn imod(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_mod) {
            Some(ref f) => {
                let x = Object::Interface(self);
                env.call(f,&x,&[y.clone()])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }

    fn rimod(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_rmod) {
            Some(ref f) => {
                env.call(f,&x,&[Object::Interface(self)])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }

    fn pow(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_pow) {
            Some(ref f) => {
                let x = Object::Interface(self);
                env.call(f,&x,&[y.clone()])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }

    fn rpow(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_rpow) {
            Some(ref f) => {
                env.call(f,&x,&[Object::Interface(self)])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }

    fn lt(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_lt) {
            Some(ref f) => {
                let x = Object::Interface(self);
                env.call(f,&x,&[y.clone()])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }
    
    fn rlt(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_rlt) {
            Some(ref f) => {
                env.call(f,&x,&[Object::Interface(self)])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }

    fn le(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_le) {
            Some(ref f) => {
                let x = Object::Interface(self);
                env.call(f,&x,&[y.clone()])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }
    
    fn rle(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_rle) {
            Some(ref f) => {
                env.call(f,&x,&[Object::Interface(self)])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }

    fn eq(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_eq) {
            Some(ref f) => {
                let x = Object::Interface(self);
                env.call(f,&x,&[y.clone()])
            },
            None => {
                Ok(Object::unimplemented())
            }
        }
    }

    fn req(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_req) {
            Some(ref f) => {
                env.call(f,&x,&[Object::Interface(self)])
            },
            None => {
                Ok(Object::Bool(match x {
                    Object::Interface(px) => ptr_eq_plain(&self,px),
                    _ => false
                }))
            }
        }
    }

    fn is_instance_of(&self, type_obj: &Object, _rte: &RTE) -> bool {
        let t = match downcast::<Table>(type_obj) {
            Some(t) => t,
            None => return false
        };
        let mut p = &self.prototype;
        loop{
            if let Some(pt) = downcast::<Table>(p) {
                if pt as *const Table == t as *const Table {
                    return true;
                }else{
                    p = &pt.prototype;
                }
            }else{
                return false;
            }
        }
    }

    fn get(self: Rc<Self>, key: &Object, env: &mut Env) -> FnResult {
        if let Some(value) = table_get(&self,key) {
            return Ok(value);
        }else{
            let key = key.clone().string(env)?;
            return env.index_error(&format!(
                "Index error in t.{0}: '{0}' not in property chain.", key
            ));
        }
    }

    fn set(self: Rc<Self>, env: &mut Env, key: Object, value: Object) -> FnResult {
        let mut m = self.map.borrow_mut();
        if m.frozen {
            return Err(env.value_error_plain("Value error in 'a.x = value': a is frozen."));
        }
        match m.m.insert(key,value) {
            Some(_) => {},
            None => {}
        }
        Ok(Object::Null)
    }
    
    fn index(self: Rc<Self>, indices: &[Object], env: &mut Env) -> FnResult {
        if let Some(f) = self.slot(&env.rte().key_index) {
            return env.call(&f,&Object::Interface(self),indices);
        }else{
            return env.type_error1(
                "Type error in x[i]: x is not indexable.","x",&Object::Interface(self));
        }
    }

    fn abs(self: Rc<Self>, env: &mut Env) -> FnResult {
        if let Some(f) = self.slot(&env.rte().key_abs) {
            return env.call(&f,&Object::Interface(self),&[]);
        }else{
            return env.std_exception(
                "Error: abs(x) is not implemented for objects of this type.");
        }
    }

    fn get_type(&self, _env: &mut Env) -> FnResult {
        Ok(if let Some(pt) = downcast::<Tuple>(&self.prototype) {
            pt.v[TYPE].clone()
        }else{
            self.prototype.clone()
        })
    }

    fn iter(self: Rc<Self>, env: &mut Env) -> FnResult {
        match self.slot(&env.rte().key_iter) {
            Some(ref iter) => {
                env.call(iter,&Object::Interface(self),&[])
            },
            None => {
                env.type_error("Type error in iter(x): x is not iterable.")
            }
        }  
    }
}

pub fn table_get(mut p: &Table, key: &Object) -> Option<Object> {
    loop{
        if let Some(y) = p.map.borrow().m.get(key) {
            return Some(y.clone());
        }else{
            if let Some(pt) = downcast::<Table>(&p.prototype) {
                p = pt;
            }else{
                match p.prototype {
                    Object::List(ref a) => {
                        return list_get(a,key);
                    },
                    Object::Interface(ref x) => {
                        if let Some(x) = x.as_any().downcast_ref::<Tuple>() {
                            if let Some(prototype) = x.v.get(PROTOTYPE) {
                                return object_get(prototype,key);
                            }else{
                                return None;
                            }
                        }
                        return None;
                    },
                    _ => return None
                }
            }
        }
    }
}

fn list_get(a: &Rc<RefCell<List>>, key: &Object) -> Option<Object> {
    for x in &a.borrow().v {
        if let Some(pt) = downcast::<Table>(x) {
            if let Some(y) = table_get(pt,key) {
                return Some(y.clone());
            }
        }
    }
    return None;
}

pub fn type_get(prototype: &Object, key: &Object) -> Option<Object> {
    if let Object::Interface(x) = prototype {
        let p = x.as_any();
        if let Some(pt) = p.downcast_ref::<Table>() {
            table_get(pt,key)
        }else if let Some(t) = p.downcast_ref::<Tuple>() {
            let ty = &t.v[TYPE];
            if let Object::Null = ty {
                object_get(&t.v[PROTOTYPE],key)
            }else{
                object_get(ty,key)
            }
        }else{
            None
        }
    }else{
        None
    }
}

pub fn object_get(x: &Object, key: &Object) -> Option<Object> {
    if let Some(pt) = downcast::<Table>(x) {
        table_get(pt,key)
    }else if let Object::List(ref a) = *x {
        list_get(a,key)
    }else{
        None
    }
}

pub fn type_to_string(_env: &mut Env, pself: &Object, _argv: &[Object])
-> FnResult
{
    if let Some(pt) = downcast::<Table>(pself) {
        if let Some(t) = downcast::<Tuple>(&pt.prototype) {
            if let Some(s) = t.v.get(NAME) {
                return Ok(s.clone());
            }
        }
    }
    return Ok(Object::Null);
}

