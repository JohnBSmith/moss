
extern crate moss;
use std::env;

fn main(){
  let argv: Vec<String> = env::args().collect();
  if argv.len()<=1 {
    moss::command_line_session();  
  }else{
    moss::eval_file(&argv[1]);
  }
}

