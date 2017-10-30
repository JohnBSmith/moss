
use std::rc::Rc;
use std::cell::RefCell;
use complex::Complex64;
use std::collections::HashMap;

pub enum Object{
  Null, Bool(bool), Int(i32), Float(f64), Complex(Complex64),
  List(Rc<RefCell<List>>), String(Rc<U32String>),
  Map(Rc<RefCell<Map>>), Function(Rc<Function>)
}

impl Object{
  pub fn clone(x: &Object) -> Object{
    match x {
      &Object::Null => {Object::Null},
      &Object::Bool(x) => {Object::Bool(x)},
      &Object::Int(x) => {Object::Int(x)},
      &Object::Float(x) => {Object::Float(x)},
      &Object::Complex(x) => {Object::Complex(x)},
      &Object::String(ref x) => {Object::String(x.clone())},
      &Object::List(ref x) => {Object::List(x.clone())},
      &Object::Map(ref x) => {Object::Map(x.clone())},
      &Object::Function(ref x) => {Object::Function(x.clone())}
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
}

pub struct List{
  pub v: Vec<Object>
}

impl List{
  pub fn new_object(v: Vec<Object>) -> Object{
    return Object::List(Rc::new(RefCell::new(List{v: v})));
  }
}

pub struct Map{
  pub m: HashMap<Object,Object>
}

impl Map{
  pub fn new_object(m: HashMap<Object,Object>) -> Object{
    return Object::Map(Rc::new(RefCell::new(Map{m: m})));
  }
  pub fn new() -> Rc<RefCell<Map>>{
    return Rc::new(RefCell::new(Map{m: HashMap::new()}));
  }
  pub fn insert_str(&mut self, key: &str, value: Object){
    self.m.insert(U32String::new_object_str(key),value);
  }
}

pub enum Function{
  Plain(PlainFn)
}

impl Function{
  pub fn plain(fp: PlainFn) -> Object {
    Object::Function(Rc::new(Function::Plain(fp)))
  }
}

pub struct Exception{
}

pub type FnResult = Result<(),Box<Exception>>;
pub type PlainFn = fn(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult;
