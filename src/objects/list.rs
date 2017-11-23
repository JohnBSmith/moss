
#[allow(unused_imports)]
use object::{Object, FnResult, U32String, Function, Table, List,
  type_error, argc_error, index_error
};
use vm::Env;

fn push(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"push");
  }
  match *pself {
    Object::List(ref a) => {
      let mut a = a.borrow_mut();
      a.v.push(argv[0].clone());
      Ok(())
    },
    _ => type_error("Type error in a.push(x): a is not a list.")
  }
}

fn map(env: &mut Env, ret: &mut Object,
  pself: &Object, argv: &[Object]
) -> FnResult
{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"map");
  }
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => {return type_error("Type error in a.map(f): a is not a list.");}
  };
  let mut v: Vec<Object> = Vec::with_capacity(a.v.len());
  for x in &a.v {
    let mut y = Object::Null;
    try!(env.call(&argv[0],&mut y,&Object::Null,&[x.clone()]));
    v.push(y);
  }
  *ret = List::new_object(v);
  return Ok(());
}

pub fn init(t: &Table){
  let mut m = t.map.borrow_mut();
  m.insert("push",Function::plain(push,1,1));
  m.insert("map", Function::env(map,1,1));
}
