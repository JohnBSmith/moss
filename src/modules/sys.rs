
#![allow(unused_imports)]

use std::rc::Rc;
use std::process;
use object::{Object, FnResult, Function,
  type_error, argc_error, new_module
};
use vm::RTE;

fn exit(pself: &Object, argv: &[Object]) -> FnResult {
  match argv.len() {
    0 => {
      process::exit(0);
    },
    1 => {
      let x = match argv[0] {
        Object::Int(x)=>x,
        _ => return type_error("Type error in exit(n): n is not an integer.")
      };
      process::exit(x);
    },
    n => {
      argc_error(n,0,1,"exit")
    }
  }
}

pub fn load_sys(rte: &Rc<RTE>) -> Object {
  let sys = new_module("sys");
  {
    let mut m = sys.map.borrow_mut();
    if let Some(ref argv) = *rte.argv.borrow() {
      m.insert("argv", Object::List(argv.clone()));
    }
    m.insert_fn_plain("exit",exit,0,1);
    m.insert_fn_env  ("call",::vm::sys_call,1,1);
  }
  return Object::Table(Rc::new(sys));
}
