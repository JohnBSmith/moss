
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use std::io;
use std::io::Write;

#[path = "compiler/compiler.rs"]
mod compiler;

fn main(){
  loop{
    let mut input = String::new();
    print!("> ");
    io::stdout().flush().ok();
    match io::stdin().read_line(&mut input) {
      Ok(_) => {},
      Err(error) => {println!("Error: {}", error);},
    };
    input.pop();
    if input=="quit" {break}
    // println!("input: '{}'",input);
    match compiler::scan(&input) {
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


