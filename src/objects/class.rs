
use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::mem::replace;

use crate::object::{
    Object, Map, Interface, downcast, ptr_eq_plain,
    FnResult, Exception
};
use crate::vm::{Env,RTE,secondary_env,object_to_string};

type PGet = Box<dyn Fn(&mut Env,Rc<Instance>,&Object)->FnResult>;
type PSet = Box<dyn Fn(&mut Env,Rc<Instance>,Object,Object)->FnResult>;
type PToString = Box<dyn Fn(&mut Env,Rc<Instance>)->Result<String,Box<Exception>>>;

pub struct Class {
    pub rte: Rc<RTE>,
    pub call_drop: bool,
    pub destructor: Object,
    pub pget: PGet,
    pub pset: PSet,
    pub to_string: PToString,
    pub name: String,
    pub map: Rc<RefCell<Map>>,
    pub parent: Object
}

impl Class {
    pub fn destructor(&self, mut t: Instance, env: &mut Env){
        if let Object::Function(_) = self.destructor {
            let p = Rc::new(t);
            {
                let tobj = Object::Interface(p.clone());
                env.call(&self.destructor, &tobj, &[]).unwrap();
            }
            if let Ok(mut t) = Rc::try_unwrap(p) {
                t.prototype = Object::Null;
            }
        }else{
            t.prototype = Object::Null;
        }
    }
}

impl Interface for Class {
    fn as_any(&self) -> &dyn Any {self}
    fn type_name(&self, _env: &mut Env) -> String {
        "Class".to_string()
    }
    fn to_string(self: Rc<Self>, _env: &mut Env) -> Result<String,Box<Exception>> {
        Ok(self.name.clone())
    }
    fn get(self: Rc<Self>, key: &Object, _env: &mut Env) -> FnResult {
        if let Some(value) = self.map.borrow().m.get(key) {
            return Ok(value.clone());
        }else{
            return Ok(Object::Null);
        }
    }
    fn set(self: Rc<Self>, _env: &mut Env, key: Object, value: Object) -> FnResult {
        self.map.borrow_mut().m.insert(key,value);
        return Ok(Object::Null);
    }
    fn eq_plain(&self, b: &Object) -> bool {
        match downcast::<Class>(b) {Some(_) => true, None => false}
    }
    fn eq(self: Rc<Self>, b: &Object, _env: &mut Env) -> FnResult {
        Ok(Object::Bool(match downcast::<Class>(b) {
            Some(_) => true, None => false
        }))
    }
}

fn get_from_ancestor(mut p: &Object, key: &Object) -> Option<Object> {
    loop{
        if let Some(class) = downcast::<Class>(p) {
            if let Some(y) = class.map.borrow().m.get(key) {
                return Some(y.clone());
            }else{
                p = &class.parent;
                if let Object::Null = p {return None;}
            }
        }else if let Object::List(a) = p {
            for x in &a.borrow().v {
                if let Some(value) = get_from_ancestor(x,key) {
                    return Some(value);
                }
            }
            return None;
        }else{
            return None;
        }
    }
}

fn standard_getter(env: &mut Env, t: Rc<Instance>, key: &Object)
-> FnResult
{
    if let Some(y) = t.map.borrow().m.get(key) {
        return Ok(y.clone());
    }else{
        let mut p = &t.prototype;
        loop{
            if let Some(class) = downcast::<Class>(p) {
                if let Some(y) = class.map.borrow().m.get(key) {
                    return Ok(y.clone());
                }else{
                    p = &class.parent;
                    if let Object::Null = p {break;}
                }
            }else if let Object::List(a) = p {
                for x in &a.borrow().v {
                    if let Some(value) = get_from_ancestor(x,key) {
                        return Ok(value);
                    }
                }
                break;
            }else{
                break;
            }
        }
        return env.index_error("Index error: slot not found.");
    }
}

fn custom_getter(f: Object) -> PGet {
    Box::new(move |env: &mut Env, t: Rc<Instance>, key: &Object| -> FnResult {
        env.call(&f,&Object::Interface(t.clone()),&[key.clone()])
    })
}

fn standard_setter(_env: &mut Env, t: Rc<Instance>, key: Object, value: Object)
-> FnResult
{
    t.map.borrow_mut().m.insert(key,value);
    return Ok(Object::Null);
}

fn custom_setter(f: Object) -> PSet {
    Box::new(move |env: &mut Env, t: Rc<Instance>, key: Object, value: Object| -> FnResult {
        env.call(&f,&Object::Interface(t.clone()),&[key,value])
    })
}

fn standard_to_string(env: &mut Env, t: Rc<Instance>)
-> Result<String,Box<Exception>>
{
    match object_to_string(env,&Object::Map(t.map.clone())) {
        Ok(value) => {
            let mut s = String::from("table");
            s.push_str(&value);
            Ok(s)
        },
        Err(e) => Err(e)
    }
}

fn custom_to_string(f: Object) -> PToString {
    Box::new(move |env: &mut Env, t: Rc<Instance>|
    -> Result<String,Box<Exception>>
    {
        match env.call(&f,&Object::Interface(t),&[]) {
            Ok(value) => match value {
                Object::String(s) => Ok(s.to_string()),
                _ => {
                    return Err(env.type_error_plain(
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
    let mut destructor = Object::Null;
    let mut call_drop = false;
    let name: String = argv[0].to_string();
    let parent: Object = argv[1].clone();

    match argv[2] {
        Object::Map(ref map) => {
            let m = &map.borrow().m;
            if let Some(x) = m.get(&Object::from("drop")) {
                destructor = x.clone();
                call_drop = true;
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
            return Ok(Object::Interface(Rc::new(Class{
                rte: env.rte().clone(),
                destructor, call_drop, pget, pset,
                name, to_string, map: map.clone(),
                parent
            })));
        },
        _ => panic!()
    }
}

pub struct Instance {
    pub prototype: Object,
    pub map: Rc<RefCell<Map>>
}

impl Instance {
    pub fn slot(&self, key: &Object) -> Option<Object> {
        if let Some(class) = downcast::<Class>(&self.prototype) {
            match class.map.borrow_mut().m.get(key) {
                Some(value) => Some(value.clone()),
                None => None
            }
        }else{
            None
        }
    }
}

impl Interface for Instance {
    fn as_any(&self) -> &dyn Any {self}
    fn get_type(&self, _env: &mut Env) -> FnResult {
        return Ok(self.prototype.clone());
    }

    fn to_string(self: Rc<Self>, env: &mut Env) -> Result<String,Box<Exception>> {
        if let Some(class) = downcast::<Class>(&self.prototype) {
            (class.to_string)(env,self.clone())
        }else{
            Ok(String::from("interface object"))
        }
    }
    fn is_instance_of(&self, type_obj: &Object, _rte: &RTE) -> bool {
        if let Object::Interface(t) = type_obj {
            if let Object::Interface(p) = &self.prototype {
                return ptr_eq_plain(p,t); 
            }
        }
        return false;
    }
    fn get(self: Rc<Self>, key: &Object, env: &mut Env) -> FnResult {
        if let Some(class) = downcast::<Class>(&self.prototype) {
            (class.pget)(env,self.clone(),key)
        }else{
            unreachable!();
        }
    }
    fn set(self: Rc<Self>, env: &mut Env, key: Object, value: Object) -> FnResult {
        if let Some(class) = downcast::<Class>(&self.prototype) {
            (class.pset)(env,self.clone(),key,value)
        }else{
            unreachable!();
        }        
    }
    fn neg(self: Rc<Self>, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_neg) {
            env.call(f,&Object::Interface(self),&[])
        }else{
            env.type_error1("Type error in -x.","x",&Object::Interface(self))
        }
    }
    fn add(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_add) {
            env.call(f,&Object::Interface(self),&[y.clone()])
        }else{
            Ok(Object::unimplemented())
        }
    }
    fn radd(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_radd) {
            env.call(f,&x,&[Object::Interface(self)])
        }else{
            Ok(Object::unimplemented())
        }
    }
    fn sub(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_sub) {
            env.call(f,&Object::Interface(self),&[y.clone()])
        }else{
            Ok(Object::unimplemented())
        }
    }
    fn rsub(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_rsub) {
            env.call(f,&x,&[Object::Interface(self)])
        }else{
            Ok(Object::unimplemented())
        }
    }
    fn mul(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_mul) {
            env.call(f,&Object::Interface(self),&[y.clone()])
        }else{
            Ok(Object::unimplemented())
        }
    }
    fn rmul(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_rmul) {
            env.call(f,&x,&[Object::Interface(self)])
        }else{
            Ok(Object::unimplemented())
        }
    }
    fn div(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_div) {
            env.call(f,&Object::Interface(self),&[y.clone()])
        }else{
            Ok(Object::unimplemented())
        }
    }
    fn rdiv(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_rdiv) {
            env.call(f,&x,&[Object::Interface(self)])
        }else{
            Ok(Object::unimplemented())
        }
    }
    fn pow(self: Rc<Self>, y: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_pow) {
            env.call(f,&Object::Interface(self),&[y.clone()])
        }else{
            Ok(Object::unimplemented())
        }
    }
    fn rpow(self: Rc<Self>, x: &Object, env: &mut Env) -> FnResult {
        if let Some(ref f) = self.slot(&env.rte().key_rpow) {
            env.call(f,&x,&[Object::Interface(self)])
        }else{
            Ok(Object::unimplemented())
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        if let Some(class) = downcast::<Class>(&self.prototype) {
            if !class.call_drop {return;}
            if class.rte.root_drop.get() {
                let state = &mut class.rte.secondary_state.borrow_mut();
                let env = &mut secondary_env(&class.rte,state);
                let t = Instance {
                    prototype: self.prototype.clone(),
                    map: replace(&mut self.map, class.rte.empty_map.clone())
                };
                class.rte.root_drop.set(false);
                class.destructor(t,env);
                loop{
                    let x = class.rte.drop_buffer.borrow_mut().pop();
                    if let Some(t) = x {
                        class.destructor(t,env);
                    }else{
                        break;
                    }
                }
                class.rte.root_drop.set(true);
            }else{
                let buffer = &mut class.rte.drop_buffer.borrow_mut();
                buffer.push(Instance {
                    prototype: self.prototype.clone(),
                    map: replace(&mut self.map, class.rte.empty_map.clone())
                });
            }
        }
    }
}

