
use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::mem::replace;

use crate::object::{
    Object, List, Map, Interface, downcast, ptr_eq_plain,
    FnResult, Exception
};
use crate::vm::{Env, RTE, secondary_env, object_to_string};

type PGet = Box<dyn Fn(&mut Env, Rc<Table>, &Object) -> FnResult>;
type PSet = Box<dyn Fn(&mut Env, Rc<Table>, Object, Object) -> FnResult>;
type PToString = Box<dyn Fn(&mut Env, Rc<Table>) -> Result<String,Box<Exception>>>;

pub struct Destructor {
    pub rte: Rc<RTE>,
    pub destructor: Object
}

impl Destructor {
    pub fn destructor(&self, mut t: Table, env: &mut Env){
        if let Object::Function(_) = self.destructor {
            let p = Rc::new(t);
            {
                let tobj = Object::Interface(p.clone());
                env.call(&self.destructor, &tobj, &[]).unwrap();
            }
            if let Ok(mut t) = Rc::try_unwrap(p) {
                t.prototype = Object::Null;
            }
        } else {
            t.prototype = Object::Null;
        }
    }
}

pub struct Class {
    pub drop: Option<Destructor>,
    pub pget: PGet,
    pub pset: PSet,
    pub to_string: PToString,
    pub name: String,
    pub map: Rc<RefCell<Map>>,
    pub parent: Object
}

impl Class {
    pub fn new(name: &str, parent: &Object) -> Rc<Class> {
        Rc::new(Class {
            drop: None,
            pget: Box::new(standard_getter),
            pset: Box::new(standard_setter),
            to_string: Box::new(standard_to_string),
            name: String::from(name),
            map: Map::new(),
            parent: parent.clone()
        })
    }
    pub fn slot(&self, key: &Object) -> Option<Object> {
        match self.map.borrow().m.get(key) {
            Some(value) => Some(value.clone()),
            None => None
        }
    }
}

impl Interface for Class {
    fn as_any(&self) -> &dyn Any {self}
    fn get_type(&self, env: &mut Env) -> FnResult {
        Ok(Object::Interface(env.rte().type_type.clone()))
    }
    fn is_instance_of(&self, type_obj: &Object, rte: &RTE) -> bool {
        if let Object::Interface(t) = type_obj {
            return ptr_eq_plain(t, &rte.type_type);
        }
        false
    }
    fn type_name(&self, _env: &mut Env) -> String {
        "Class".to_string()
    }
    fn to_string(self: Rc<Self>, _env: &mut Env) -> Result<String,Box<Exception>> {
        Ok(self.name.clone())
    }
    fn get(self: Rc<Self>, key: &Object, _env: &mut Env) -> FnResult {
        Ok(if let Some(value) = self.map.borrow().m.get(key) {
            value.clone()
        } else {
            Object::Null
        })
    }
    fn set(self: Rc<Self>, _env: &mut Env, key: Object, value: Object) -> FnResult {
        self.map.borrow_mut().m.insert(key, value);
        Ok(Object::Null)
    }
    fn eq_plain(&self, b: &Object) -> bool {
        downcast::<Class>(b).is_some()
    }
    fn eq(self: Rc<Self>, b: &Object, _env: &mut Env) -> FnResult {
        Ok(Object::Bool(downcast::<Class>(b).is_some()))
    }
}

fn get_from_ancestor(mut p: &Object, key: &Object) -> Option<Object> {
    loop {
        if let Some(class) = downcast::<Class>(p) {
            if let Some(y) = class.map.borrow().m.get(key) {
                return Some(y.clone());
            } else {
                p = &class.parent;
                if let Object::Null = p {return None;}
            }
        } else if let Object::List(a) = p {
            for x in &a.borrow().v {
                if let Some(value) = get_from_ancestor(x,key) {
                    return Some(value);
                }
            }
            return None;
        } else if let Some(t) = downcast::<Table>(p) {
            if let Some(y) = t.map.borrow().m.get(key) {
                return Some(y.clone());
            } else {
                p = &t.prototype;
                if let Object::Null = p {return None;}
            }
        } else {
            return None;
        }
    }
}

fn standard_getter(env: &mut Env, t: Rc<Table>, key: &Object)
-> FnResult
{
    if let Some(y) = t.map.borrow().m.get(key) {
        Ok(y.clone())
    } else {
        let mut p = &t.prototype;
        loop {
            if let Some(class) = downcast::<Class>(p) {
                if let Some(y) = class.map.borrow().m.get(key) {
                    return Ok(y.clone());
                } else {
                    p = &class.parent;
                    if let Object::Null = p {break;}
                }
            } else if let Object::List(a) = p {
                for x in &a.borrow().v {
                    if let Some(value) = get_from_ancestor(x,key) {
                        return Ok(value);
                    }
                }
                break;
            } else if let Some(t) = downcast::<Table>(p) {
                if let Some(y) = t.map.borrow().m.get(key) {
                    return Ok(y.clone());
                } else {
                    p = &t.prototype;
                    if let Object::Null = p {break;}
                }
            } else {
                break;
            }
        }
        env.index_error(&format!("Index error: slot '{}' not found.", key))
    }
}

fn custom_getter(f: Object) -> PGet {
    Box::new(move |env: &mut Env, t: Rc<Table>, key: &Object| -> FnResult {
        env.call(&f, &Object::Interface(t), &[key.clone()])
    })
}

fn standard_setter(_env: &mut Env, t: Rc<Table>, key: Object, value: Object)
-> FnResult
{
    t.map.borrow_mut().m.insert(key, value);
    Ok(Object::Null)
}

fn custom_setter(f: Object) -> PSet {
    Box::new(move |env: &mut Env, t: Rc<Table>, key: Object, value: Object| -> FnResult {
        env.call(&f, &Object::Interface(t), &[key, value])
    })
}

fn standard_to_string(env: &mut Env, t: Rc<Table>)
-> Result<String,Box<Exception>>
{
    match object_to_string(env,&Object::Map(t.map.clone())) {
        Ok(value) => {
            let mut s = String::from("table");
            if let Some(class) = downcast::<Class>(&t.prototype) {
                s.push(' ');
                s.push_str(&class.name);
            }
            s.push_str(&value);
            Ok(s)
        },
        Err(e) => Err(e)
    }
}

fn custom_to_string(f: Object) -> PToString {
    Box::new(move |env: &mut Env, t: Rc<Table>|
    -> Result<String,Box<Exception>>
    {
        match env.call(&f, &Object::Interface(t), &[]) {
            Ok(value) => match value {
                Object::String(s) => Ok(s.to_string()),
                _ => {
                    Err(env.type_error_plain(
                        "Type error in x.string(): return value is not a string."))
                }
            },
            Err(e) => Err(e)
        }
    })
}

pub fn class_new(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        3 => {}, n => return env.argc_error(n,3,3,"class")
    }
    let mut drop = None;
    let name: String = argv[0].to_string();
    let parent: Object = argv[1].clone();

    match argv[2] {
        Object::Map(ref map) => {
            let m = &map.borrow().m;
            if let Some(x) = m.get(&Object::from("drop")) {
                drop = Some(Destructor{
                    rte: env.rte().clone(),
                    destructor: x.clone()
                });
            }
            let pget: PGet = match m.get(&Object::from("get")) {
                Some(f) => custom_getter(f.clone()),
                None => Box::new(standard_getter)
            };
            let pset: PSet = match m.get(&Object::from("set")) {
                Some(f) => custom_setter(f.clone()),
                None => Box::new(standard_setter)
            };
            let to_string: PToString = match m.get(&env.rte().key_string) {
                Some(f) => custom_to_string(f.clone()),
                None => Box::new(standard_to_string)
            };
            Ok(Object::Interface(Rc::new(Class {
                drop, pget, pset, name, to_string,
                map: map.clone(), parent
            })))
        },
        _ => panic!()
    }
}

pub struct Table {
    pub prototype: Object,
    pub map: Rc<RefCell<Map>>
}

impl Table {
    pub fn new(prototype: Object) -> Rc<Table> {
        Rc::new(Table{prototype, map: Map::new()})
    }
    pub fn slot(&self, key: &Object) -> Option<Object> {
        if let Some(class) = downcast::<Class>(&self.prototype) {
            match class.map.borrow_mut().m.get(key) {
                Some(value) => Some(value.clone()),
                None => None
            }
        } else {
            None
        }
    }
}

impl Interface for Table {
    fn as_any(&self) -> &dyn Any {self}
    fn get_type(&self, _env: &mut Env) -> FnResult {
        Ok(self.prototype.clone())
    }

    fn to_string(self: Rc<Self>, env: &mut Env) -> Result<String,Box<Exception>> {
        if let Some(class) = downcast::<Class>(&self.prototype) {
            (class.to_string)(env,self.clone())
        } else {
            standard_to_string(env,self.clone())
        }
    }
    fn is_instance_of(&self, type_obj: &Object, _rte: &RTE) -> bool {
        if let Object::Interface(t) = type_obj {
            let mut pobj = &self.prototype;
            loop {
                if let Object::Interface(p) = pobj {
                    if ptr_eq_plain(p,t) {return true;}
                    if let Some(pclass) = p.as_any().downcast_ref::<Class>() {
                        pobj = &pclass.parent;
                    } else if let Some(pt) = p.as_any().downcast_ref::<Table>() {
                        pobj = &pt.prototype;
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }
        false
    }
    fn get(self: Rc<Self>, key: &Object, env: &mut Env) -> FnResult {
        if let Some(class) = downcast::<Class>(&self.prototype) {
            (class.pget)(env, self.clone(), key)
        } else {
            standard_getter(env, self.clone(), key)
        }
    }
    fn set(self: Rc<Self>, env: &mut Env, key: Object, value: Object) -> FnResult {
        if let Some(class) = downcast::<Class>(&self.prototype) {
            (class.pset)(env, self.clone(), key, value)
        } else {
            standard_setter(env, self.clone(), key, value)
        }
    }
    fn neg(self: Rc<Self>, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_neg) {
            env.call(f, &Object::Interface(self), &[])
        } else {
            env.type_error1("Type error in -x.", "x", &Object::Interface(self))
        }
    }
    fn add(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_add) {
            env.call(f, &Object::Interface(self), &[y.clone()])
        } else {
            Ok(Object::unimplemented())
        }
    }
    fn radd(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_radd) {
            env.call(f, &x, &[Object::Interface(self)])
        } else {
            Ok(Object::unimplemented())
        }
    }
    fn sub(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_sub) {
            env.call(f, &Object::Interface(self), &[y.clone()])
        } else {
            Ok(Object::unimplemented())
        }
    }
    fn rsub(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_rsub) {
            env.call(f, &x, &[Object::Interface(self)])
        } else {
            Ok(Object::unimplemented())
        }
    }
    fn mul(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_mul) {
            env.call(f, &Object::Interface(self), &[y.clone()])
        } else {
            Ok(Object::unimplemented())
        }
    }
    fn rmul(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_rmul) {
            env.call(f, &x, &[Object::Interface(self)])
        } else {
            Ok(Object::unimplemented())
        }
    }
    fn div(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_div) {
            env.call(f, &Object::Interface(self), &[y.clone()])
        } else {
            Ok(Object::unimplemented())
        }
    }
    fn rdiv(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_rdiv) {
            env.call(f, &x, &[Object::Interface(self)])
        } else {
            Ok(Object::unimplemented())
        }
    }
    fn pow(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_pow) {
            env.call(f, &Object::Interface(self), &[y.clone()])
        } else {
            Ok(Object::unimplemented())
        }
    }
    fn rpow(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_rpow) {
            env.call(f, &x, &[Object::Interface(self)])
        } else {
            Ok(Object::unimplemented())
        }
    }
    fn lt(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_lt) {
            env.call(f, &Object::Interface(self), &[y.clone()])
        } else {
            Ok(Object::unimplemented())
        }
    }
    fn rlt(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_rlt) {
            env.call(f, &x, &[Object::Interface(self)])
        } else {
            Ok(Object::unimplemented())
        }
    }
    fn le(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_le) {
            env.call(f, &Object::Interface(self), &[y.clone()])
        } else {
            Ok(Object::unimplemented())
        }
    }
    fn rle(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_rle) {
            env.call(f, &x, &[Object::Interface(self)])
        } else {
            Ok(Object::unimplemented())
        }
    }
    fn eq(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_eq) {
            env.call(f, &Object::Interface(self), &[y.clone()])
        } else {
            Ok(Object::unimplemented())
        }
    }
    fn req(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_req) {
            env.call(f, &x, &[Object::Interface(self)])
        } else {
            Ok(Object::Bool(match x {
                Object::Interface(px) => ptr_eq_plain(&self,px),
                _ => false
            }))
        }
    }
    fn index(self: Rc<Self>, indices: &[Object], env: &mut Env) -> FnResult {
        if let Some(f) = self.slot(&env.rte().key_index) {
            env.call(&f, &Object::Interface(self), indices)
        } else {
            env.type_error1(
                "Type error in x[i]: x is not indexable.", "x",
                &Object::Interface(self))
        }
    }
    fn iter(self: Rc<Self>, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_iter) {
            env.call(f, &Object::Interface(self), &[])
        } else {
            env.type_error1("Type error in iter(x).", "x",
                &Object::Interface(self))
        }
    }
    fn call(&self, env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_call) {
            env.call(f, pself, argv)
        } else {
            env.type_error1("Type error in x(...).", "x", pself)
        }
    }
    fn abs(self: Rc<Self>, env: &mut Env) -> FnResult {
        if let Some(f) = self.slot(&env.rte().key_abs) {
            env.call(&f, &Object::Interface(self), &[])
        } else {
            env.std_exception(
                "Error: abs(x) is not implemented for objects of this type.")
        }
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        if let Some(class) = downcast::<Class>(&self.prototype) {
            if let Some(context) = &class.drop {
                if context.rte.root_drop.get() {
                    let state = &mut context.rte.secondary_state.borrow_mut();
                    let env = &mut secondary_env(&context.rte, state);
                    let t = Table {
                        prototype: self.prototype.clone(),
                        map: replace(&mut self.map, context.rte.empty_map.clone())
                    };
                    context.rte.root_drop.set(false);
                    context.destructor(t,env);
                    loop {
                        let x = context.rte.drop_buffer.borrow_mut().pop();
                        if let Some(t) = x {
                            context.destructor(t,env);
                        } else {
                            break;
                        }
                    }
                    context.rte.root_drop.set(true);
                } else {
                    let buffer = &mut context.rte.drop_buffer.borrow_mut();
                    buffer.push(Table {
                        prototype: self.prototype.clone(),
                        map: replace(&mut self.map, context.rte.empty_map.clone())
                    });
                }
            }
        }
    }
}

pub fn table_get(mut p: &Table, key: &Object) -> Option<Object> {
    loop {
        if let Some(y) = p.map.borrow().m.get(key) {
            return Some(y.clone());
        } else if let Some(pt) = downcast::<Table>(&p.prototype) {
            p = pt;
        } else if let Some(pc) = downcast::<Class>(&p.prototype) {
            return class_get(pc,key);
        } else if let Object::List(ref a) = p.prototype {
            return list_get(a,key);
        } else {
            return None;
        }
    }
}

fn list_get(a: &Rc<RefCell<List>>, key: &Object) -> Option<Object> {
    for x in &a.borrow().v {
        if let Some(pt) = downcast::<Table>(x) {
            if let Some(y) = table_get(pt,key) {
                return Some(y);
            }
        }
    }
    None
}

fn class_get(c: &Class, key: &Object) -> Option<Object> {
    if let Some(value) = c.map.borrow().m.get(key) {
        Some(value.clone())
    } else {
        object_get(&c.parent, key)
    }
}

pub fn object_get(x: &Object, key: &Object) -> Option<Object> {
    if let Some(pt) = downcast::<Table>(x) {
        table_get(pt,key)
    } else if let Object::List(ref a) = *x {
        list_get(a,key)
    } else {
        None
    }
}

