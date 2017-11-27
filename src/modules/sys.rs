
#![allow(unused_imports)]

use std::rc::Rc;
use object::{Object, FnResult, Function,
  type_error, argc_error, new_module
};
use vm::RTE;

pub fn load_sys(rte: &Rc<RTE>) -> Object {
  let sys = new_module("sys");
  {
    let mut m = sys.map.borrow_mut();
    if let Some(ref argv) = *rte.argv.borrow() {
      m.insert("argv", Object::List(argv.clone()));
    }
  }
  return Object::Table(Rc::new(sys));
}
