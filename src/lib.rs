
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

pub const STACK_SIZE: usize = 4000;
pub const FRAME_STACK_SIZE: usize = 200;

// Assert size of usize is at least 32 bit.

#[path = "system/system.rs"]
mod system;

pub mod object;
mod compiler;
mod vm;
mod global;

#[path = "objects/list.rs"]
mod list;

#[path = "objects/map.rs"]
mod map;

#[path = "objects/function.rs"]
mod function;

#[path = "objects/iterable.rs"]
mod iterable;

#[path = "objects/string.rs"]
mod string;

#[path = "objects/long-gmp.rs"]
mod long;

#[path = "modules/complex.rs"]
pub mod complex;

#[path = "modules/math.rs"]
mod math;

#[path = "modules/rand.rs"]
pub mod rand;

#[path = "modules/sys.rs"]
mod sys;

#[path = "modules/la.rs"]
mod la;

#[path = "modules/sf.rs"]
mod sf;

#[path = "modules/format.rs"]
mod format;

use std::rc::Rc;
use std::cell::RefCell;
// use std::fs::File;
// use std::io::Read;
use object::{Object, List, Map, U32String, FnResult};
use vm::{RTE,State,EnvPart,Frame, get_env};

pub struct Interpreter{
  pub rte: Rc<RTE>,
  pub state: RefCell<State>
}

impl Interpreter{
  pub fn new_config(stack_size: usize) -> Self {
    let rte = RTE::new();
    ::global::init_rte(&rte);

    let mut stack: Vec<Object> = Vec::with_capacity(stack_size);
    for _ in 0..stack_size {
      stack.push(Object::Null);
    }
    
    let mut frame_stack: Vec<Frame> = Vec::with_capacity(FRAME_STACK_SIZE);
    let mut state = RefCell::new(State{
      stack: stack, sp: 0,
      env: EnvPart::new(frame_stack, rte.clone())
    });

    return Self{rte, state};
  }

  pub fn new() -> Self {
    Interpreter::new_config(STACK_SIZE)
  }

  pub fn eval_env(&self, s: &str, gtab: Rc<RefCell<Map>>) -> Object {
    let mut state = &mut *self.state.borrow_mut();
    let mut env = get_env(&mut state);
    return env.eval_env(s,gtab);
  }
  
  pub fn eval_file(&self, id: &str, gtab: Rc<RefCell<Map>>){
    let mut state = &mut *self.state.borrow_mut();
    let mut env = get_env(&mut state);
    return env.eval_file(id,gtab);
  }
  
  pub fn eval(&self, s: &str) -> Object {
    let mut state = &mut *self.state.borrow_mut();
    let mut env = get_env(&mut state);
    return env.eval(s);  
  }

  pub fn command_line_session(&self, gtab: Rc<RefCell<Map>>){
    let mut state = &mut *self.state.borrow_mut();
    let mut env = get_env(&mut state);
    env.command_line_session(gtab);
  }

  pub fn call(&self, f: &Object, pself: &Object, argv: &[Object]) -> FnResult {
    let mut state = &mut *self.state.borrow_mut();
    let mut env = get_env(&mut state);
    return env.call(f,pself,argv);
  }
}

pub fn new_list_str(a: &[String]) -> Rc<RefCell<List>> {
  let mut v: Vec<Object> = Vec::with_capacity(a.len());
  for i in 0..a.len() {
    v.push(U32String::new_object_str(&a[i]));
  }
  return Rc::new(RefCell::new(List{v: v, frozen: false}));
}

impl Drop for Interpreter {
  fn drop(&mut self) {
    let v = self.rte.delay.borrow_mut();
    for gtab in &v[..] {
      // println!("clear {}",Object::Map(gtab.clone()));
      gtab.borrow_mut().m.clear();
    }
  }
}

