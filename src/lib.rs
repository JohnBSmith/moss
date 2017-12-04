
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

#[path = "objects/map.rs"]
mod map;

#[path = "objects/function.rs"]
mod function;

#[path = "objects/iterable.rs"]
mod iterable;

#[path = "modules/complex.rs"]
pub mod complex;

#[path = "modules/math.rs"]
mod math;

#[path = "modules/rand.rs"]
pub mod rand;

#[path = "modules/sys.rs"]
mod sys;

use std::rc::Rc;
use std::cell::RefCell;
// use std::fs::File;
// use std::io::Read;
use object::{Object, List, Map, U32String};
use vm::{RTE,State,EnvPart,Frame,STACK_SIZE,FRAME_STACK_SIZE,
  get_env
};

pub struct Interpreter{
  pub rte: Rc<RTE>,
  pub state: RefCell<State>
}

impl Interpreter{
  pub fn new() -> Self {
    let rte = RTE::new();
    ::global::init_rte(&rte);

    let mut stack: Vec<Object> = Vec::with_capacity(STACK_SIZE);
    for _ in 0..STACK_SIZE {
      stack.push(Object::Null);
    }
    let mut frame_stack: Vec<Frame> = Vec::with_capacity(FRAME_STACK_SIZE);
    let mut state = RefCell::new(State{
      stack: stack, sp: 0,
      env: EnvPart::new(frame_stack, rte.clone())
    });

    return Self{rte, state};
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

  pub fn command_line_session(&self, gtab: Rc<RefCell<Map>>){
    let mut state = &mut *self.state.borrow_mut();
    let mut env = get_env(&mut state);
    env.command_line_session(gtab);
  }
}

pub fn new_list_str(a: &[String]) -> Rc<RefCell<List>> {
  let mut v: Vec<Object> = Vec::with_capacity(a.len());
  for i in 0..a.len() {
    v.push(U32String::new_object_str(&a[i]));
  }
  return Rc::new(RefCell::new(List{v: v, frozen: false}));
}
