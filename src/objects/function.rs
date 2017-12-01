
#![allow(unused_imports)]

use std::rc::Rc;
use std::cell::RefCell;
use object::{Object, FnResult, U32String, Function, Table, List,
  type_error, argc_error, index_error,
  VARIADIC, MutableFn, EnumFunction
};
use vm::Env;

fn apply(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()==1 {
    match argv[0] {
      Object::List(ref a) => {
        env.call(pself,&Object::Null,&a.borrow().v)
      },
      _ => type_error("Type error in f.apply(a): a is not a list.")
    }
  }else if argv.len()==2 {
    match argv[1] {
      Object::List(ref a) => {
        env.call(pself,&argv[0],&a.borrow().v)
      },
      _ => type_error("Type error in f.apply(a): a is not a list.")    
    }
  }else{
    return argc_error(argv.len(),1,1,"apply");
  }
}

fn orbit(pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()!=1 {
    return argc_error(argv.len(),1,1,"orbit");
  }
  let mut x = argv[0].clone();
  let f = pself.clone();
  let i = Box::new(
    move |env: &mut Env, pself: &Object, argv: &[Object]| -> FnResult {
      let y = x.clone();
      x = try!(env.call(&f,&Object::Null,&[x.clone()]));
      return Ok(y);
    }
  );
  return Ok(Object::Function(Rc::new(Function{
    f: EnumFunction::Env(RefCell::new(i)),
    argc: 0, argc_min: 0, argc_max: 0,
    id: Object::Null
  })));
}

pub fn init(t: &Table){
  let mut m = t.map.borrow_mut();
  m.insert_fn_env  ("apply",apply,1,1);
  m.insert_fn_plain("orbit",orbit,1,1);
}
