

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

#[cfg(feature = "long-none")]
#[path = "objects/long-none.rs"]
mod long;

#[path = "objects/tuple.rs"]
mod tuple;

#[path = "objects/class.rs"]
mod class;

#[path = "modules/module.rs"]
pub mod module;

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

#[path = "modules/time.rs"]
mod time;

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

#[path = "modules/fs.rs"]
mod fs;

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
use object::{Object, List, CharString, TypeName, Downcast};
use vm::{RTE,State,EnvPart,Env};
pub use vm::{get_env};
pub use compiler::{Value, CompilerExtra};

pub struct InterpreterLock<'a> {
    state: std::cell::RefMut<'a,State>
}

impl<'a> InterpreterLock<'a> {
    pub fn env(&mut self) -> Env {
        return get_env(&mut self.state)
    }
}

pub struct Interpreter{
    pub rte: Rc<RTE>,
    pub state: RefCell<State>
}

impl Interpreter{
    pub fn lock(&self) -> InterpreterLock {
        InterpreterLock{state: self.state.borrow_mut()}
    }
    pub fn tie<T>(&self, f: impl FnOnce(&mut Env)->T) -> T {
        f(&mut self.lock().env())
    }
    pub fn eval(&self, s: &str) -> Object {
        self.lock().env().eval(s)
    }
    pub fn eval_cast<T>(&self, s: &str) -> T
    where T: TypeName+Downcast<Output=T>
    {
        self.tie(|env| {
           let y = env.eval(s);
           env.downcast::<T>(&y)
        })
    }

    pub fn new_config(stack_size: usize) -> Self {
        let rte = RTE::new();
        ::global::init_rte(&rte);

        let mut stack: Vec<Object> = Vec::with_capacity(stack_size);
        for _ in 0..stack_size {
            stack.push(Object::Null);
        }

        let state = RefCell::new(State{
            stack: stack, sp: 0,
            env: EnvPart::new(FRAME_STACK_SIZE, rte.clone())
        });

        return Self{rte, state};
    }

    pub fn new() -> Rc<Self> {
        Rc::new(Interpreter::new_config(STACK_SIZE))
    }

    pub fn repr(&self, x: &Object) -> String {
        let mut ilock = self.lock();
        let mut env = ilock.env();
        match x.repr(&mut env) {
            Ok(s)=>s,
            Err(e)=>{
                println!("{}",env.exception_to_string(&e));
                "[exception in Interpreter::repr, see stdout]".to_string()
            }
        }
    }

    pub fn string(&self, x: &Object) -> String {
        let mut ilock = self.lock();
        let mut env = ilock.env();
        match x.string(&mut self.lock().env()) {
            Ok(s) => return s,
            Err(e) => {
                println!("{}",env.exception_to_string(&e));
                panic!();
            }
        }
    }

    pub fn set_config(&self, config: CompilerExtra) {
        let mut conf = self.rte.compiler_config.borrow_mut();
        *conf = Some(Box::new(config));
    }
    
    pub fn set_capabilities(&self, root_mode: bool) {
        if root_mode {
            let mut capabilities = self.rte.capabilities.borrow_mut();
            capabilities.write = true;
            capabilities.command = true;
        }
    }
}

pub fn new_list_str(a: &[String]) -> Rc<RefCell<List>> {
    let mut v: Vec<Object> = Vec::with_capacity(a.len());
    for i in 0..a.len() {
        v.push(CharString::new_object_str(&a[i]));
    }
    return Rc::new(RefCell::new(List{v: v, frozen: false}));
}

impl Drop for Interpreter {
    fn drop(&mut self) {
        let v = self.rte.delay.borrow_mut();
        let mut buffer: Vec<Object> = Vec::new();
        for gtab in &v[..] {
            // println!("clear {}",Object::Map(gtab.clone()));
            {
                let m = &mut gtab.borrow_mut().m;
                for (_k,v) in m.drain() {
                    buffer.push(v);
                }
            }
            buffer.clear();
        }
    }
}

