
#![allow(unused_imports)]

use std::rc::Rc;
use std::cell::{Cell,RefCell};
use std::collections::HashMap;
use std::fmt;
use std::any::Any;
use std::ops;
use std::fmt::Write;

use complex::Complex64;
use vm::{Module,Env};
use global::type_name;

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
  Tuple(Rc<Vec<Object>>),
  Empty,
  Interface(Rc<Interface>)
}

impl Object{
  pub fn to_string(&self) -> String {
    ::vm::object_to_string(self)
  }
  pub fn repr(&self) -> String {
    ::vm::object_to_repr(self)
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
      Object::Tuple(ref x) => {Object::Tuple(x.clone())},
      Object::Table(ref x) => {Object::Table(x.clone())},
      Object::Empty => {Object::Empty},
      Object::Interface(ref x) => {Object::Interface(x.clone())}
    }
  }
}

impl fmt::Display for Object {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", ::vm::object_to_string(self))
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
  pub fn insert_fn_env(&mut self, key: &str, fp: EnvFn, argc_min: u32, argc_max: u32){
    let key = U32String::new_object_str(key);

    let f = Object::Function(Rc::new(Function{
      f: EnumFunction::Env(fp),
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
  pub col: usize
}

pub struct Exception{
  pub value: Object,
  pub traceback: Option<List>,
  pub spot: Option<Spot>
}

impl Exception{
  pub fn new(s: &str) -> Box<Exception> {
    Box::new(Exception{
      value: U32String::new_object_str(s),
      traceback: None, spot: None
    })
  }
  pub fn set_spot(&mut self, line: usize, col: usize) {
    self.spot = Some(Spot{line,col});
  }
}

pub fn std_exception_plain(s: &str) -> Box<Exception> {
  Exception::new(s)
}
pub fn type_error_plain(s: &str) -> Box<Exception> {
  Exception::new(s)
}
pub fn value_error_plain(s: &str) -> Box<Exception> {
  Exception::new(s)
}
pub fn index_error_plain(s: &str) -> Box<Exception> {
  Exception::new(s)
}

#[inline(never)]
pub fn std_exception(s: &str) -> FnResult {
  Err(Exception::new(s))
}

#[inline(never)]
pub fn type_error(s: &str) -> FnResult {
  Err(Exception::new(s))
}

#[inline(never)]
pub fn value_error(s: &str) -> FnResult {
  Err(Exception::new(s))
}

#[inline(never)]
pub fn index_error(s: &str) -> FnResult {
  Err(Exception::new(s))
}

pub fn argc_error_plain(argc: usize, min: u32, max: u32, id: &str) -> Box<Exception> {
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

#[inline(never)]
pub fn argc_error(argc: usize, min: u32, max: u32, id: &str) -> FnResult {
  Err(argc_error_plain(argc,min,max,id))
}

fn bounded_repr(x: &Object) -> String {
  let s = x.repr();
  if s.len()>32 {
    return s[0..32].to_string()+"... ";
  }else{
    return s;
  }
}

#[inline(never)]
pub fn type_error1_plain(
  s: &str, sx: &str, x: &Object
) -> Box<Exception>
{
  let mut buffer = s.to_string();
  write!(buffer,"\nNote:\n").unwrap();
  write!(buffer,"  {0}: {1}, {0} = {2}.",sx,&type_name(x),&bounded_repr(x)).unwrap();
  return type_error_plain(&buffer);
}

#[inline(never)]
pub fn type_error1(
  s: &str, sx: &str, x: &Object
) -> FnResult
{
  return Err(type_error1_plain(s,sx,x));
}

#[inline(never)]
pub fn type_error2_plain(
  s: &str, sx: &str, sy: &str, x: &Object, y: &Object
) -> Box<Exception>
{
  let mut buffer = s.to_string();
  write!(buffer,"\nNote:\n").unwrap();
  write!(buffer,"  {0}: {1}, {0} = {2},\n",sx,&type_name(x),&bounded_repr(x)).unwrap();
  write!(buffer,"  {0}: {1}, {0} = {2}.",sy,&type_name(y),&bounded_repr(y)).unwrap();
  return type_error_plain(&buffer);
}

#[inline(never)]
pub fn type_error2(
  s: &str, sx: &str, sy: &str, x: &Object, y: &Object
) -> FnResult
{
  return Err(type_error2_plain(s,sx,sy,x,y));
}

pub type OperatorResult = Result<(),Box<Exception>>;
pub type FnResult = Result<Object,Box<Exception>>;

pub type PlainFn = fn(pself: &Object, argv: &[Object]) -> FnResult;
pub type EnvFn = fn(&mut Env, pself: &Object, argv: &[Object]) -> FnResult;
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
  Env(EnvFn),
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
  pub fn env(fp: EnvFn, argc_min: u32, argc_max: u32) -> Object {
    Object::Function(Rc::new(Function{
      f: EnumFunction::Env(fp),
      argc: if argc_min==argc_max {argc_min} else {VARIADIC},
      argc_min: argc_min, argc_max: argc_max,
      id: Object::Null
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

pub struct Table{
  pub prototype: Object,
  pub map: Rc<RefCell<Map>>
}
impl Table{
  pub fn new(prototype: Object) -> Rc<Table> {
    Rc::new(Table{prototype: prototype, map: Map::new()})
  }
}

pub fn new_module(id: &str) -> Table{
  Table{prototype: Object::Null, map: Map::new()}
}

pub trait Interface{
  fn as_any(&self) -> &Any;
  fn to_string(&self) -> String {
    "interface object".to_string()
  }
  fn add(&self, b: &Object, &mut Env) -> FnResult {
    std_exception("Error: a+b is not implemented for objects of this type.")
  }
  fn radd(&self, a: &Object, &mut Env) -> FnResult {
    std_exception("Error: a+b is not implemented for objects of this type.")
  }
  fn sub(&self, b: &Object, &mut Env) -> FnResult {
    std_exception("Error: a-b is not implemented for objects of this type.")
  }
  fn rsub(&self, a: &Object, &mut Env) -> FnResult {
    std_exception("Error: a-b is not implemented for objects of this type.")
  }
  fn mpy(&self, b: &Object, &mut Env) -> FnResult {
    std_exception("Error: a*b is not implemented for objects of this type.")
  }
  fn rmpy(&self, a: &Object, &mut Env) -> FnResult {
    std_exception("Error: a*b is not implemented for objects of this type.")
  }
  fn div(&self, b: &Object, &mut Env) -> FnResult {
    std_exception("Error: a/b is not implemented for objects of this type.")
  }
  fn rdiv(&self, a: &Object, &mut Env) -> FnResult {
    std_exception("Error: a/b is not implemented for objects of this type.")
  }
  fn idiv(&self, b: &Object, &mut Env) -> FnResult {
    std_exception("Error: a//b is not implemented for objects of this type.")
  }
  fn ridiv(&self, a: &Object, &mut Env) -> FnResult {
    std_exception("Error: a//b is not implemented for objects of this type.")
  }
  fn imod(&self, b: &Object, &mut Env) -> FnResult {
    std_exception("Error: a%b is not implemented for objects of this type.")
  }
  fn pow(&self, b: &Object, &mut Env) -> FnResult {
    std_exception("Error: a^b is not implemented for objects of this type.")
  }
  fn rpow(&self, b: &Object, &mut Env) -> FnResult {
    std_exception("Error: a^b is not implemented for objects of this type.")
  }

  fn eq(&self, b: &Object) -> bool {
    false
  }
  fn req(&self, a: &Object) -> bool {
    false
  }
  fn lt(&self, b: &Object, &mut Env) -> FnResult {
    std_exception("Error: a<b is not implemented for objects of this type.")
  }
  fn gt(&self, b: &Object, &mut Env) -> FnResult {
    std_exception("Error: a>b is not implemented for objects of this type.")
  }
  fn le(&self, b: &Object, &mut Env) -> FnResult {
    std_exception("Error: a<=b is not implemented for objects of this type.")
  }
  fn ge(&self, b: &Object, &mut Env) -> FnResult {
    std_exception("Error: a>=b is not implemented for objects of this type.")
  }

  fn rlt(&self, b: &Object, &mut Env) -> FnResult {
    std_exception("Error: a<b is not implemented for objects of this type.")
  }
  fn rgt(&self, b: &Object, &mut Env) -> FnResult {
    std_exception("Error: a>b is not implemented for objects of this type.")
  }
  fn rle(&self, b: &Object, &mut Env) -> FnResult {
    std_exception("Error: a<=b is not implemented for objects of this type.")
  }
  fn rge(&self, b: &Object, &mut Env) -> FnResult {
    std_exception("Error: a>=b is not implemented for objects of this type.")
  }
  
  fn neg(&self, &mut Env) -> FnResult {
    std_exception("Error: -a is not implemented for objects of this type.")
  }
  
  fn abs(&self) -> FnResult {
    std_exception("Error: abs(x) is not implemented for objects of this type.")
  }
  fn type_name(&self) -> String {
    "Interface object".to_string()
  }
}

