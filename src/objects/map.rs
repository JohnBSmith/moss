
#![allow(unused_imports)]

use object::{Object, FnResult, U32String, Function, Table, List, Map,
  type_error, argc_error, index_error, std_exception,
  VARIADIC, MutableFn
};

pub fn update(m: &mut Map, m2: &Map){
  for (key,value) in &m2.m {
    m.m.insert(key.clone(),value.clone());
  }
}

fn fupdate(pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()!=1 {
    return argc_error(argv.len(),1,1,"update");
  }
  match *pself {
    Object::Map(ref m) => {
      match argv[0] {
        Object::Map(ref m2) => {
          update(&mut *m.borrow_mut(),&*m2.borrow());
          Ok(Object::Null)
        },
        _ => type_error("Type error in m.update(m2): m2 is not a map.")
      }
    },
    _ => type_error("Type error in m.update(m2): m is not a map.")
  }
}

pub fn init(t: &Table){
  let mut m = t.map.borrow_mut();
  m.insert("update", Function::plain(fupdate,1,1));
}
