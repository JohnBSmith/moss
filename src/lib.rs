

pub const STACK_SIZE: usize = 4000;
pub const FRAME_STACK_SIZE: usize = 200;

// Assert size of usize is at least 32 bit.

#[path = "system/system.rs"]
mod system;

#[macro_use]
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

#[cfg(feature = "long-gmp")]
#[path = "objects/long-gmp.rs"]
mod long;

#[cfg(all(not(feature = "long"),not(feature = "long-gmp")))]
#[path = "objects/long-none.rs"]
mod long;

#[path = "objects/tuple.rs"]
mod tuple;

#[path = "modules/complex.rs"]
pub mod complex;

#[path = "modules/math.rs"]
mod math;

#[path = "modules/rand.rs"]
pub mod rand;

#[path = "modules/format.rs"]
mod format;

#[path = "modules/sys.rs"]
mod sys;

#[cfg(feature = "la")]
#[path = "modules/la.rs"]
mod la;

#[cfg(feature = "math-la")]
#[path = "modules/math-la.rs"]
mod math_la;

#[cfg(feature = "math-sf")]
#[path = "modules/sf.rs"]
mod sf;

#[path = "modules/regex.rs"]
mod regex;

#[path = "modules/data.rs"]
mod data;

#[cfg(feature = "gx")]
#[path = "modules/sdl.rs"]
mod sdl;

#[cfg(feature = "gx")]
#[path = "modules/gx.rs"]
mod gx;

use std::rc::Rc;
use std::cell::RefCell;
// use std::fs::File;
// use std::io::Read;
use object::{Object, List, Map, U32String, FnResult, Exception};
use vm::{RTE,State,EnvPart,Frame, get_env};
pub use compiler::{Value, CompilerExtra};

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

        let frame_stack: Vec<Frame> = Vec::with_capacity(FRAME_STACK_SIZE);
        let state = RefCell::new(State{
            stack: stack, sp: 0,
            env: EnvPart::new(frame_stack, rte.clone())
        });

        return Self{rte, state};
    }

    pub fn new() -> Self {
        Interpreter::new_config(STACK_SIZE)
    }
    
    pub fn eval_string(&self, s: &str, id: &str,
        gtab: Rc<RefCell<Map>>, value: compiler::Value
    ) -> Result<Object,Box<Exception>>
    {
        let mut state = &mut *self.state.borrow_mut();
        let mut env = get_env(&mut state);
        return env.eval_string(s,id,gtab,value);
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

    pub fn call(&self, f: &Object, pself: &Object, argv: &[Object])
    -> FnResult
    {
        let mut state = &mut *self.state.borrow_mut();
        let mut env = get_env(&mut state);
        return env.call(f,pself,argv);
    }

    pub fn repr(&self, x: &Object) -> String {
        let mut state = &mut *self.state.borrow_mut();
        let mut env = get_env(&mut state);
        match x.repr(&mut env) {
            Ok(s)=>s,
            Err(e)=>{
                println!("{}",env.exception_to_string(&e));
                "[exception in Interpreter::repr, see stdout]".to_string()
            }
        }
    }

    pub fn string(&self, x: &Object) -> String {
        let mut state = &mut *self.state.borrow_mut();
        let mut env = get_env(&mut state);
        match x.string(&mut env) {
            Ok(s) => return s,
            Err(e) => {
                println!("{}",env.exception_to_string(&e));
                panic!();
            }
        }
    }

    pub fn exception_to_string(&self, e: &Exception) -> String {
        let mut state = &mut *self.state.borrow_mut();
        let mut env = get_env(&mut state);
        return env.exception_to_string(e);
    }

    pub fn print_exception(&self, e: &Exception) {
        println!("{}",self.exception_to_string(e));
    }

    pub fn print_type_and_value(&self, x: &Object) {
        let mut state = &mut *self.state.borrow_mut();
        let mut env = get_env(&mut state);
        env.print_type_and_value(x);
    }
    
    pub fn set_config(&self, config: CompilerExtra) {
        let mut conf = self.rte.compiler_config.borrow_mut();
        *conf = Some(Box::new(config));
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

