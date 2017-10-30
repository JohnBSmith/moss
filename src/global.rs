
use vm::Object;
use vm::object_to_string;

pub fn print(x: &mut Object, pself: &Object, argv: &[Object]) -> bool{
  println!("{}",object_to_string(&argv[0]));
  return false;
}
