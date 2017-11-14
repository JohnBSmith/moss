
#[allow(unused_imports)]
use object::{Object, FnResult, U32String, Function, Table,
  type_error, argc_error, index_error
};

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

pub fn init(t: &Table){
  let mut m = t.map.borrow_mut();
  m.insert("push",Function::plain(push,1,1));
}
