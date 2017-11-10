
use std::rc::Rc;
use std::cell::RefCell;
use complex::Complex64;
use std::collections::HashMap;
use vm::Module;

pub enum Object{
  Null, Bool(bool), Int(i32), Float(f64), Complex(Complex64),
  List(Rc<RefCell<List>>), String(Rc<U32String>),
  Map(Rc<RefCell<Map>>), Function(Rc<Function>),
  Range(Rc<Range>)
}

impl Object{
  pub fn clone(&self) -> Object{
    match *self {
      Object::Null => {Object::Null},
      Object::Bool(x) => {Object::Bool(x)},
      Object::Int(x) => {Object::Int(x)},
      Object::Float(x) => {Object::Float(x)},
      Object::Complex(x) => {Object::Complex(x)},
      Object::String(ref x) => {Object::String(x.clone())},
      Object::List(ref x) => {Object::List(x.clone())},
      Object::Map(ref x) => {Object::Map(x.clone())},
      Object::Function(ref x) => {Object::Function(x.clone())},
      Object::Range(ref x) => {Object::Range(x.clone())}
    }
  }
  pub fn str(&self) -> String {
    ::vm::object_to_string(self)
  }
}

pub struct U32String{
  pub v: Vec<char>
}

impl U32String{
  pub fn new_object(v: Vec<char>) -> Object{
    return Object::String(Rc::new(U32String{v: v}));
  }
  pub fn new_object_str(s: &str) -> Object{
    return Object::String(Rc::new(U32String{v: s.chars().collect()}));
  }
}

pub struct List{
  pub v: Vec<Object>,
  pub frozen: bool
}

impl List{
  pub fn new_object(v: Vec<Object>) -> Object{
    return Object::List(Rc::new(RefCell::new(List{v: v, frozen: false})));
  }
}

pub struct Map{
  pub m: HashMap<Object,Object>,
  pub frozen: bool
}

impl Map{
  pub fn new_object(m: HashMap<Object,Object>) -> Object{
    return Object::Map(Rc::new(RefCell::new(Map{m: m, frozen: false})));
  }
  pub fn new() -> Rc<RefCell<Map>>{
    return Rc::new(RefCell::new(Map{m: HashMap::new(), frozen: false}));
  }
  pub fn insert(&mut self, key: &str, value: Object){
    self.m.insert(U32String::new_object_str(key),value);
  }
}

pub struct Range{
  pub a: Object,
  pub b: Object,
  pub step: Object
}

pub struct Exception{
  pub value: Object,
  pub traceback: Option<List>
}

impl Exception{
  pub fn new(s: &str) -> FnResult {
    Err(Box::new(Exception{
      value: U32String::new_object_str(s),
      traceback: None
    }))
  }
}

pub fn type_error(s: &str) -> FnResult{
  Exception::new(s)
}

pub fn index_error(s: &str) -> FnResult{
  Exception::new(s)
}

pub fn argc_error(argc: usize, min: u32, max: u32, id: &str) -> FnResult{
  if min==max {
    if min==0 {
      Exception::new(&format!("Error in {}: expected no argument, but got {}.",id,argc))
    }else if min==1 {
      Exception::new(&format!("Error in {}: expected 1 argument, but got {}.",id,argc))    
    }else{
      Exception::new(&format!("Error in {}: expected {} arguments, but got {}.",id,min,argc))    
    }
  }else{
    Exception::new(&format!("Error in {}: expected {}..{} arguments, but got {}.",id,min,max,argc))
  }
}

pub type FnResult = Result<(),Box<Exception>>;
pub type PlainFn = fn(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult;
pub struct StandardFn{
  pub address: usize,
  pub module: Rc<Module>,
  pub gtab: Rc<RefCell<Map>>,
  pub var_count: u32
}

pub enum EnumFunction{
  Plain(PlainFn),
  Std(StandardFn)
}

pub struct Function{
  pub f: EnumFunction,
  pub argc: u32,
  pub argc_min: u32,
  pub argc_max: u32
}

pub const VARIADIC: u32 = 0xffffffff;

impl Function{
  pub fn plain(fp: PlainFn, argc_min: u32, argc_max: u32) -> Object {
    Object::Function(Rc::new(Function{
      f: EnumFunction::Plain(fp),
      argc: if argc_min==argc_max {argc_min} else {VARIADIC},
      argc_min: argc_min, argc_max: argc_max
    }))
  }
  pub fn new(f: StandardFn, argc_min: u32, argc_max: u32) -> Object {
    Object::Function(Rc::new(Function{
      f: EnumFunction::Std(f),
      argc: if argc_min==argc_max {argc_min} else {VARIADIC},
      argc_min: argc_min, argc_max: argc_max
    }))
  }
}

