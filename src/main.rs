
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

#[path = "compiler/compiler.rs"]
mod compiler;

#[path = "system/system.rs"]
mod system;

fn _main(){
  loop{
    let s = system::getline("# ").unwrap();
    println!("[{}]",s);
  }
}


fn main(){
  loop{
    let mut input = String::new();
    match system::getline("> ") {
      Ok(s) => {input=s;},
      Err(error) => {println!("Error: {}", error);},
    };
    if input=="quit" {break}
    match compiler::scan(&input,1) {
      Ok(v) => {
        // compiler::print_vtoken(&v);
        match compiler::compile(v,true) {
          Ok(_) => {},
          Err(e) => {compiler::print_syntax_error(e);}
        };
      },
      Err(error) => {
        compiler::print_syntax_error(error);
      }
    }
  }
}
