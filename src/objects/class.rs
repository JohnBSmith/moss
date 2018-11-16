
use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::mem::replace;

use object::{Object,Map,Interface,FnResult,Exception,downcast};
use vm::{Env,RTE,secondary_env,object_to_string};

type PGet = Box<dyn Fn(&mut Env,&Instance,&Object)->FnResult>;
type PSet = Box<dyn Fn(&mut Env,&Instance,Object,Object)->FnResult>;
type PToString = Box<dyn Fn(&mut Env,&Instance)->Result<String,Box<Exception>>>;

pub struct Class{
    pub rte: Rc<RTE>,
    pub call_drop: bool,
    pub destructor: Object,
    pub pget: PGet,
    pub pset: PSet,
    pub to_string: PToString,
    pub name: String
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
    fn as_any(&self) -> &Any {self}
    fn type_name(&self) -> String {
        "Class".to_string()
    }
    fn to_string(&self, _env: &mut Env) -> Result<String,Box<Exception>> {
        Ok(self.name.clone())
    }
}

fn standard_getter(env: &mut Env, t: &Instance, key: &Object)
-> FnResult
{
    if let Some(y) = t.map.borrow().m.get(key) {
        return Ok(y.clone());
    }else{
        return env.index_error("Index error: slot not found.");
    }
}

fn custom_getter(f: Object) -> PGet {
    Box::new(move |env: &mut Env, t: &Instance, key: &Object| -> FnResult {
        env.call(&f,&Object::Map(t.map.clone()),&[key.clone()])
    })
}

fn standard_setter(_env: &mut Env, t: &Instance, key: Object, value: Object)
-> FnResult
{
    t.map.borrow_mut().m.insert(key,value);
    return Ok(Object::Null);
}

fn custom_setter(f: Object) -> PSet {
    Box::new(move |env: &mut Env, t: &Instance, key: Object, value: Object| -> FnResult {
        env.call(&f,&Object::Map(t.map.clone()),&[key,value])
    })
}

fn standard_to_string(env: &mut Env, t: &Instance)
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
    Box::new(move |env: &mut Env, t: &Instance|
    -> Result<String,Box<Exception>>
    {
        match env.call(&f,&Object::Map(t.map.clone()),&[]) {
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
        1 => {}, n => return env.argc_error(n,1,1,"class")
    }
    let mut destructor = Object::Null;
    let mut call_drop = false;
    match argv[0] {
        Object::Map(ref m) => {
            let m = &m.borrow().m;
            if let Some(x) = m.get(&Object::from("drop")) {
                destructor = x.clone();
                call_drop = true;
            }
            let name: String = match m.get(&Object::from("name")) {
                Some(value) => value.to_string(),
                None => String::from("class object")
            };
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
                name, to_string
            })));
        },
        _ => panic!()
    }
}

pub struct Instance{
    pub prototype: Object,
    pub map: Rc<RefCell<Map>>
}

impl Interface for Instance {
    fn as_any(&self) -> &Any {self}
    fn get_type(&self, _env: &mut Env) -> FnResult {
        return Ok(self.prototype.clone());
    }
    
    // needing self: &Rc<Self>
    fn to_string(&self, env: &mut Env) -> Result<String,Box<Exception>> {
        if let Some(class) = downcast::<Class>(&self.prototype) {
            (class.to_string)(env,self)
        }else{
            Ok(String::from("interface object"))
        }
    }
    fn is_instance_of(&self, type_obj: &Object, _rte: &RTE) -> bool {
        if let Object::Interface(t) = type_obj {
            if let Object::Interface(p) = &self.prototype {
                return Rc::ptr_eq(p,t);
            }
        }
        return false;
    }
    fn get(&self, key: &Object, env: &mut Env) -> FnResult {
        if let Some(class) = downcast::<Class>(&self.prototype) {
            (class.pget)(env,self,key)
        }else{
            unreachable!();
        }
    }
    fn set(&self, env: &mut Env, key: Object, value: Object) -> FnResult {
        if let Some(class) = downcast::<Class>(&self.prototype) {
            (class.pset)(env,self,key,value)
        }else{
            unreachable!();
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
                    if let Some(mut t) = x {
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

