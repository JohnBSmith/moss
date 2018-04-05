
#![allow(unused_imports)]

use std::rc::Rc;
use std::cell::{Cell,RefCell};
use std::collections::HashMap;
use std::fmt;
use std::any::Any;
use std::ops;

use complex::Complex64;
use vm::{Module,RTE};
pub use vm::Env;

pub enum Object{
  Null,
  Bool(bool),
  Int(i32),
  Float(f64),
  Complex(Complex64),

  List(Rc<RefCell<List>>),
  String(Rc<U32String>),
  Map(Rc<RefCell<Map>>),
  Function(Rc<Function>),
  Range(Rc<Range>),
  Table(Rc<Table>),
  Interface(Rc<Interface>),
  Empty
}

impl Object{
  pub fn string(&self, env: &mut Env) -> Result<String,Box<Exception>> {
    ::vm::object_to_string(env,self)
  }
  pub fn repr(&self, env: &mut Env) -> Result<String,Box<Exception>> {
    ::vm::object_to_repr(env,self)
  }
  pub fn to_repr(&self) -> String {
    ::vm::object_to_repr_plain(self)
  }
}

impl fmt::Display for Object {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", ::vm::object_to_string_plain(self))
  }
}

impl Clone for Object{
  fn clone(&self) -> Object{
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
      Object::Range(ref x) => {Object::Range(x.clone())},
      Object::Table(ref x) => {Object::Table(x.clone())},
      Object::Empty => {Object::Empty},
      Object::Interface(ref x) => {Object::Interface(x.clone())}
    }
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
  pub fn new_object_char(c: char) -> Object{
    return Object::String(Rc::new(U32String{v: vec![c]}));
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
  pub fn new() -> Self {
    return List{v: Vec::new(), frozen: false};
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
  pub fn insert_fn_plain(&mut self, key: &str, fp: PlainFn, argc_min: u32, argc_max: u32){
    let key = U32String::new_object_str(key);

    let f = Object::Function(Rc::new(Function{
      f: EnumFunction::Plain(fp),
      argc: if argc_min==argc_max {argc_min} else {VARIADIC},
      argc_min: argc_min, argc_max: argc_max,
      id: key.clone()
    }));
    
    self.m.insert(key,f);
  }
}

pub struct Range{
  pub a: Object,
  pub b: Object,
  pub step: Object
}

pub struct Spot{
  pub line: usize,
  pub col: usize,
  pub module: String
}

pub struct Exception{
  pub value: Object,
  pub traceback: Option<List>,
  pub spot: Option<Spot>
}

impl Exception{
  pub fn new(s: &str, prototype: Object) -> Box<Exception> {
    let mut t = Table{prototype, map: Map::new(), extra: None};
    t.map.borrow_mut().insert("value", U32String::new_object_str(s));
    Box::new(Exception{
      value: Object::Table(Rc::new(t)),
      traceback: None, spot: None
    })
  }
  pub fn raise(x: Object) -> Box<Exception> {
    Box::new(Exception{
      value: x, traceback: None, spot: None
    })
  }
  pub fn set_spot(&mut self, line: usize, col: usize, module: &str) {
    self.spot = Some(Spot{line,col,module: module.to_string()});
  }
  pub fn push_clm(&mut self, line: usize, col: usize, module: &str, fid: &str) {
    let s = U32String::new_object_str(&format!(
      "{}, {}:{}:{}",fid,module,line,col
    ));
    if let Some(ref mut a) = self.traceback {
      a.v.push(s);
    }else{
      let mut a = List::new();
      a.v.push(s);
      self.traceback = Some(a);
    }
  }
  pub fn traceback_push(&mut self, fid: &str) {
    let s = U32String::new_object_str(fid);
    if let Some(ref mut a) = self.traceback {
      a.v.push(s);
    }else{
      let mut a = List::new();
      a.v.push(s);
      self.traceback = Some(a);
    }
  }
}

impl fmt::Debug for Exception {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", ::vm::object_to_string_plain(&self.value))
  }
}

#[macro_export]
macro_rules! trace_err_try {
  ($e:expr, $fid:expr) => (match $e {
    Ok(val) => val,
    Err(mut err) => {
      err.traceback_push($fid);
      return Err(err)
    }
  });
}

pub type OperatorResult = Result<(),Box<Exception>>;
pub type FnResult = Result<Object,Box<Exception>>;

pub type PlainFn = fn(&mut Env, pself: &Object, argv: &[Object]) -> FnResult;
pub type MutableFn = Box<FnMut(&mut Env, &Object, &[Object])->FnResult>;

pub struct StandardFn{
  pub address: Cell<usize>,
  pub module: Rc<Module>,
  pub gtab: Rc<RefCell<Map>>,
  pub var_count: u32,
  pub context: Rc<RefCell<List>>
}

pub enum EnumFunction{
  Std(StandardFn),
  Plain(PlainFn),
  Mut(RefCell<MutableFn>)
}

pub struct Function{
  pub f: EnumFunction,
  pub argc: u32,
  pub argc_min: u32,
  pub argc_max: u32,
  pub id: Object
}

pub const VARIADIC: u32 = 0xffffffff;

impl Function{
  pub fn plain(fp: PlainFn, argc_min: u32, argc_max: u32) -> Object {
    Object::Function(Rc::new(Function{
      f: EnumFunction::Plain(fp),
      argc: if argc_min==argc_max {argc_min} else {VARIADIC},
      argc_min: argc_min, argc_max: argc_max,
      id: Object::Null
    }))
  }
  pub fn new(f: StandardFn, id: Object, argc_min: u32, argc_max: u32) -> Object {
    Object::Function(Rc::new(Function{
      f: EnumFunction::Std(f),
      argc: if argc_min==argc_max {argc_min} else {VARIADIC},
      argc_min: argc_min, argc_max: argc_max,
      id: id
    }))
  }
  pub fn mutable(fp: MutableFn, argc_min: u32, argc_max: u32) -> Object {
    Object::Function(Rc::new(Function{
      f: EnumFunction::Mut(RefCell::new(fp)),
      argc: if argc_min==argc_max {argc_min} else {VARIADIC},
      argc_min: argc_min, argc_max: argc_max,
      id: Object::Null
    }))
  }
}

pub struct TableExtra{
  pub get: Object,
  pub set: Object
  // pub drop: Object
  // pub rte: Rc<RTE>
}

pub struct Table{
  pub prototype: Object,
  pub map: Rc<RefCell<Map>>,
  pub extra: Option<Box<TableExtra>>
}
impl Table{
  pub fn new(prototype: Object) -> Rc<Table> {
    Rc::new(Table{prototype: prototype, map: Map::new(), extra: None})
  }
  pub fn get(&self, key: &Object) -> Option<Object> {
    let mut p = self;
    loop{
      match p.map.borrow_mut().m.get(key) {
        Some(value) => {return Some(value.clone());},
        None => {
          p = match p.prototype {
            Object::Table(ref t) => t,
            _ => {return None;}
          }
        }
      }
    }
  }
}

pub fn new_module(id: &str) -> Table{
  Table{prototype: Object::Null, map: Map::new(), extra: None}
}

pub trait Interface{
  fn as_any(&self) -> &Any;
  fn to_string(&self, env: &mut Env) -> Result<String,Box<Exception>> {
    Ok("interface object".to_string())
  }
  fn add(&self, b: &Object, env: &mut Env) -> FnResult {
    Ok(Object::Table(env.rte().unimplemented.clone()))
  }
  fn radd(&self, a: &Object, env: &mut Env) -> FnResult {
    Ok(Object::Table(env.rte().unimplemented.clone()))
  }
  fn sub(&self, b: &Object, env: &mut Env) -> FnResult {
    Ok(Object::Table(env.rte().unimplemented.clone()))
  }
  fn rsub(&self, a: &Object, env: &mut Env) -> FnResult {
    Ok(Object::Table(env.rte().unimplemented.clone()))
  }
  fn mpy(&self, b: &Object, env: &mut Env) -> FnResult {
    Ok(Object::Table(env.rte().unimplemented.clone()))
  }
  fn rmpy(&self, a: &Object, env: &mut Env) -> FnResult {
    Ok(Object::Table(env.rte().unimplemented.clone()))
  }
  fn div(&self, b: &Object, env: &mut Env) -> FnResult {
    Ok(Object::Table(env.rte().unimplemented.clone()))
  }
  fn rdiv(&self, a: &Object, env: &mut Env) -> FnResult {
    Ok(Object::Table(env.rte().unimplemented.clone()))
  }
  fn idiv(&self, b: &Object, env: &mut Env) -> FnResult {
    env.std_exception("Error: a//b is not implemented for objects of this type.")
  }
  fn ridiv(&self, a: &Object, env: &mut Env) -> FnResult {
    env.std_exception("Error: a//b is not implemented for objects of this type.")
  }
  fn imod(&self, b: &Object, env: &mut Env) -> FnResult {
    Ok(Object::Table(env.rte().unimplemented.clone()))
  }
  fn rimod(&self, b: &Object, env: &mut Env) -> FnResult {
    Ok(Object::Table(env.rte().unimplemented.clone()))
  }
  fn pow(&self, b: &Object, env: &mut Env) -> FnResult {
    env.std_exception("Error: a^b is not implemented for objects of this type.")
  }
  fn rpow(&self, b: &Object, env: &mut Env) -> FnResult {
    env.std_exception("Error: a^b is not implemented for objects of this type.")
  }

  fn eq_plain(&self, b: &Object) -> bool {
    false
  }
  fn req_plain(&self, a: &Object) -> bool {
    false
  }

  fn eq(&self, b: &Object, env: &mut Env) -> FnResult {
    Ok(Object::Table(env.rte().unimplemented.clone()))
  }
  fn req(&self, a: &Object, env: &mut Env) -> FnResult {
    Ok(Object::Table(env.rte().unimplemented.clone()))
  }
  fn lt(&self, b: &Object, env: &mut Env) -> FnResult {
    env.std_exception("Error: a<b is not implemented for objects of this type.")
  }
  fn gt(&self, b: &Object, env: &mut Env) -> FnResult {
    env.std_exception("Error: a>b is not implemented for objects of this type.")
  }
  fn le(&self, b: &Object, env: &mut Env) -> FnResult {
    env.std_exception("Error: a<=b is not implemented for objects of this type.")
  }
  fn ge(&self, b: &Object, env: &mut Env) -> FnResult {
    env.std_exception("Error: a>=b is not implemented for objects of this type.")
  }

  fn rlt(&self, b: &Object, env: &mut Env) -> FnResult {
    env.std_exception("Error: a<b is not implemented for objects of this type.")
  }
  fn rgt(&self, b: &Object, env: &mut Env) -> FnResult {
    env.std_exception("Error: a>b is not implemented for objects of this type.")
  }
  fn rle(&self, b: &Object, env: &mut Env) -> FnResult {
    env.std_exception("Error: a<=b is not implemented for objects of this type.")
  }
  fn rge(&self, b: &Object, env: &mut Env) -> FnResult {
    env.std_exception("Error: a>=b is not implemented for objects of this type.")
  }
  
  fn neg(&self, env: &mut Env) -> FnResult {
    Ok(Object::Table(env.rte().unimplemented.clone()))
  }

  fn abs(&self, env: &mut Env) -> FnResult {
    env.std_exception("Error: abs(x) is not implemented for objects of this type.")
  }
  fn sgn(&self, env: &mut Env) -> FnResult {
    env.std_exception("Error: sgn(x) is not implemented for objects of this type.")
  }
  fn get(&self, key: &Object, env: &mut Env) -> FnResult {
    env.std_exception("Type error in t.x: getter is not implemented for objects of this type.")
  }
  fn index(&self, indices: &[Object], env: &mut Env) -> FnResult {
    env.std_exception("Type error in a[i]: indexing is not implemented for objects of this type.")
  }
  fn set_index(&self, indices: &[Object], value: &Object, env: &mut Env) -> FnResult {
    env.std_exception("Type error in a[i]=value: indexing is not implemented for objects of this type.")
  }
  fn type_name(&self) -> String {
    "Interface object".to_string()
  }
  fn is_instance_of(&self, type_obj: &Object, rte: &RTE) -> bool {
    false
  }
  fn hash(&self) -> u64 {
    self as *const _ as *const u8 as usize as u64
  }
}
