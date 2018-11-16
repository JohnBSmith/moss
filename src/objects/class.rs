
use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::mem::replace;

use object::{Object,Map,Interface,FnResult,downcast};
use vm::{Env,RTE,secondary_env};

type PGet = Box<dyn Fn(&mut Env,&Instance,&Object)->FnResult>;

pub struct Class{
    pub rte: Rc<RTE>,
    pub destructor: Object,
    pub pget: PGet
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

pub fn class_new(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"class")
    }
    let mut destructor = Object::Null;
    match argv[0] {
        Object::Map(ref m) => {
            if let Some(x) = m.borrow().m.get(&Object::from("drop")) {
                destructor = x.clone();
            }
            let pget: PGet = match m.borrow().m.get(&Object::from("get")) {
                Some(f) => custom_getter(f.clone()),
                None => Box::new(standard_getter)
            };
            return Ok(Object::Interface(Rc::new(Class{
                rte: env.rte().clone(),
                destructor: destructor,
                pget: pget
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
    fn get(&self, key: &Object, env: &mut Env) -> FnResult {
        if let Some(class) = downcast::<Class>(&self.prototype) {
            (class.pget)(env,self,key)
        }else{
            unreachable!();
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        if let Some(class) = downcast::<Class>(&self.prototype) {
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

