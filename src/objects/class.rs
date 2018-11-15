
use std::any::Any;
use std::rc::Rc;

use object::{Object,Table,Interface,FnResult};
use vm::{Env,RTE};

pub struct Class{
    pub rte: Rc<RTE>,
    pub destructor: Object,
}

impl Class {
    pub fn destructor(&self, mut t: Table, env: &mut Env){
        if let Object::Function(_) = self.destructor {
            let tobj = Object::Table(Rc::new(t));
            env.call(&self.destructor, &tobj, &[]).unwrap();
            if let Object::Table(t) = tobj {
                if let Ok(mut t) = Rc::try_unwrap(t) {
                    t.prototype = Object::Null;
                }
            }else{
                unreachable!();
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
            return Ok(Object::Interface(Rc::new(Class{
                rte: env.rte().clone(),
                destructor: destructor
            })));
        },
        _ => panic!()
    }
}
