
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

#[path = "system/system.rs"]
mod system;

pub mod object;
mod compiler;
mod vm;
mod global;

#[path = "modules/complex.rs"]
mod complex;

#[path = "modules/math.rs"]
mod math;

use std::fs::File;
use std::io::Read;
use object::{Object, Map, Function, Exception, VARIADIC};
use vm::eval;

fn init_gtab(gtab: &mut Map){
  gtab.insert("print",Function::plain(::global::print,0,VARIADIC));
  gtab.insert("put",Function::plain(::global::put,0,VARIADIC));
  gtab.insert("str",Function::plain(::global::fstr,1,1));
  gtab.insert("repr",Function::plain(::global::repr,1,1));
  gtab.insert("abs",Function::plain(::global::abs,1,1));
  gtab.insert("eval",Function::plain(::global::eval,1,1));
  gtab.insert("size",Function::plain(::global::size,1,1));
  gtab.insert("load",Function::plain(::global::fload,1,1));
}

pub fn command_line_session(){
  let mut history = system::History::new();
  let gtab = Map::new();
  init_gtab(&mut gtab.borrow_mut());
  loop{
    let mut input = String::new();
    match system::getline_history("> ",&history) {
      Ok(s) => {
        if s=="" {continue;}
        history.append(&s);
        input=s;
      },
      Err(error) => {println!("Error: {}", error);},
    };
    if input=="quit" {break}
    match compiler::scan(&input,1,"command line") {
      Ok(v) => {
        // compiler::print_vtoken(&v);
        match compiler::compile(v,true,&mut history,"command line") {
          Ok(module) => {
            match eval(module.clone(),gtab.clone(),true) {
              Ok(x) => {},
              Err(e) => {
                println!("{}",::vm::object_to_string(&e.value));
              }
            }
          },
          Err(e) => {compiler::print_error(&e);}
        };
      },
      Err(error) => {
        compiler::print_error(&error);
      }
    }
  }
}

pub fn eval_string(s: &str, id: &str) -> Result<Object,Box<Exception>> {
  let mut history = system::History::new();
  match compiler::scan(s,1,id) {
    Ok(v) => {
      let gtab = Map::new();
      init_gtab(&mut gtab.borrow_mut());
      match compiler::compile(v,false,&mut history,id) {
        Ok(module) => {
          return eval(module.clone(),gtab.clone(),false);
        },
        Err(e) => {compiler::print_error(&e);}
      };
    },
    Err(error) => {
      compiler::print_error(&error);
    }
  }
  return Ok(Object::Null);
}

pub fn eval_file(id: &str){
  let mut path: String = String::from(id);
  path += ".moss";
  let mut f = match File::open(&path) {
    Ok(f) => f,
    Err(e) => {
      match File::open(id) {
        Ok(f) => f,
        Err(e) => {
          println!("File '{}' not found.",id);
          return;
        }
      }
    }
  };
  let mut s = String::new();
  f.read_to_string(&mut s).expect("something went wrong reading the file");
  match eval_string(&s,id) {
    Ok(x) => {},
    Err(e) => {
      println!("{}",::vm::object_to_string(&e.value));
    }
  }
}

pub struct Interpreter{}

impl Interpreter{
  pub fn new() -> Self {
    return Self{}
  }
  pub fn eval(&self, s: &str) -> Object {
    return match eval_string(s,"") {
      Ok(x) => x,
      Err(e) => e.value
    };
  }
}
