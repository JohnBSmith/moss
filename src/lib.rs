
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

#[path = "system/system.rs"]
mod system;

pub mod object;
mod compiler;
mod vm;
mod global;

#[path = "objects/list.rs"]
mod list;

#[path = "modules/complex.rs"]
mod complex;

#[path = "modules/math.rs"]
mod math;

use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use object::{Object, Map, Function, Exception, VARIADIC};
use vm::{eval,Env};

fn init_gtab(gtab: &mut Map, env: &Env){
  gtab.insert("print",Function::plain(::global::print,0,VARIADIC));
  gtab.insert("put",  Function::plain(::global::put,0,VARIADIC));
  gtab.insert("str",  Function::plain(::global::fstr,1,1));
  gtab.insert("repr", Function::plain(::global::repr,1,1));
  gtab.insert("abs",  Function::plain(::global::abs,1,1));
  gtab.insert("eval", Function::plain(::global::eval,1,1));
  gtab.insert("size", Function::plain(::global::size,1,1));
  gtab.insert("load", Function::plain(::global::fload,1,1));
  gtab.insert("iter", Function::plain(::global::iter,1,1));
  gtab.insert("record", Function::plain(::global::record,1,1));
  gtab.insert("object", Function::plain(::global::fobject,0,2));

  let list_type = env.list.clone();
  ::list::init(&list_type);
  gtab.insert("List", Object::Table(list_type));
}

fn print_exception(e: &Exception) {
  if let Some(ref spot) = e.spot {
    println!("Line {}, col {}:",spot.line,spot.col);
  }
  println!("{}",::vm::object_to_string(&e.value));
}

pub struct Interpreter{
  pub env: Rc<Env>,
  pub gtab: Rc<RefCell<Map>>
}

impl Interpreter{
  pub fn new() -> Self {
    let env = Env::new();
    let gtab = Map::new();
    init_gtab(&mut gtab.borrow_mut(),&env);
    return Self{env, gtab};
  }
  pub fn eval(&self, s: &str) -> Object {
    let gtab = Map::new();
    return match self.eval_string(s,"",gtab) {
      Ok(x) => x,
      Err(e) => e.value
    };
  }
  pub fn eval_env(&self, s: &str, gtab: Rc<RefCell<Map>>) -> Object {
    return match self.eval_string(s,"",gtab) {
      Ok(x) => x,
      Err(e) => e.value
    };    
  }

  pub fn eval_string(&self, s: &str, id: &str, gtab: Rc<RefCell<Map>>)
    -> Result<Object,Box<Exception>>
  {
    let mut history = system::History::new();
    match compiler::scan(s,1,id,false) {
      Ok(v) => {
        match compiler::compile(v,false,&mut history,id,self) {
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

  pub fn command_line_session(&self, gtab: Rc<RefCell<Map>>){
    let mut history = system::History::new();
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
      match compiler::scan(&input,1,"command line",false) {
        Ok(v) => {
          // compiler::print_vtoken(&v);
          match compiler::compile(v,true,&mut history,"command line",self) {
            Ok(module) => {
              match eval(module.clone(),gtab.clone(),true) {
                Ok(x) => {}, Err(e) => {print_exception(&e);}
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

  pub fn eval_file(&self, id: &str, gtab: Rc<RefCell<Map>>){
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

    match self.eval_string(&s,id,gtab) {
      Ok(x) => {}, Err(e) => {print_exception(&e);}
    }
  }
}
