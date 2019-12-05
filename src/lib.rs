
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

#[path = "objects/range.rs"]
mod range;

#[cfg(feature = "long-num")]
#[path = "objects/long-num.rs"]
mod long;

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

#[cfg(feature = "graphics")]
#[path = "modules/sdl.rs"]
mod sdl;

#[cfg(feature = "graphics")]
#[path = "modules/graphics.rs"]
mod graphics;

use std::rc::Rc;
use std::cell::RefCell;
// use std::fs::File;
// use std::io::Read;
use object::{Object, List, Map, CharString, TypeName, Downcast};
use vm::{RTE,State,EnvPart,Env};
pub use vm::{get_env};
pub use compiler::{Value, CompilerExtra};
use global::init_rte;

pub struct InterpreterLock<'a> {
    state: std::cell::RefMut<'a,State>
}

impl<'a> InterpreterLock<'a> {
    pub fn env(&mut self) -> Env {
        return get_env(&mut self.state);
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
        init_rte(&rte);

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
    for x in a {
        v.push(CharString::new_object_str(x));
    }
    return Rc::new(RefCell::new(List{v: v, frozen: false}));
}

fn clear_map(buffer: &mut Vec<Object>, map: &Rc<RefCell<Map>>) {
    {
        let m = &mut map.borrow_mut().m;
        for (_k,v) in m.drain() {
            buffer.push(v);
        }
    }
    buffer.clear();
}

// We need to break cyclic structures to avert memory leaks.
// The cycles can be introduced by function objects.
impl Drop for Interpreter {
    fn drop(&mut self) {
        let v = self.rte.delay.borrow_mut();
        let mut buffer: Vec<Object> = Vec::with_capacity(32);
        for gtab in &v[..] {
            // println!("clear {}",Object::Map(gtab.clone()));
            clear_map(&mut buffer,gtab);
        }
        clear_map(&mut buffer,&self.rte.type_bool.map);
        clear_map(&mut buffer,&self.rte.type_int.map);
        clear_map(&mut buffer,&self.rte.type_long.map);
        clear_map(&mut buffer,&self.rte.type_float.map);
        clear_map(&mut buffer,&self.rte.type_complex.map);
        clear_map(&mut buffer,&self.rte.type_string.map);
        clear_map(&mut buffer,&self.rte.type_list.map);
        clear_map(&mut buffer,&self.rte.type_map.map);
        clear_map(&mut buffer,&self.rte.type_range.map);
        clear_map(&mut buffer,&self.rte.type_function.map);
        clear_map(&mut buffer,&self.rte.type_iterable.map);

        let mut state = self.rte.secondary_state.borrow_mut();
        *state = None;
    }
}

fn is_file(id: &str) -> bool {
    let metadata = match std::fs::metadata(id) {
        Ok(value) => value,
        Err(_) => return false
    };
    return metadata.file_type().is_file();
}

pub fn residual_path(id: &str) -> Option<String> {
    if is_file(id) {
        return None;
    }else{
        let mut path = system::library_path();
        path.push_str("include/");
        path.push_str(id);
        return Some(path);
    }
}
