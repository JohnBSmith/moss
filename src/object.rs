
use std::rc::Rc;
use std::cell::{Cell,RefCell};
use std::collections::HashMap;
use std::fmt;
use std::any::Any;
use std::mem::replace;

use complex::Complex64;
use vm::{Module,RTE,secondary_env};
pub use vm::Env;
use class::Class;

pub enum Object{
    Null,
    Bool(bool),
    Int(i32),
    Float(f64),
    Complex(Complex64),

    List(Rc<RefCell<List>>),
    String(Rc<CharString>),
    Map(Rc<RefCell<Map>>),
    Function(Rc<Function>),
    Range(Rc<Range>),
    Table(Rc<Table>),
    Interface(Rc<dyn Interface>),
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

    #[inline(always)]
    pub fn take(&mut self) -> Object {
        replace(self,Object::Null)
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

pub struct CharString{
    pub data: Vec<char>
}

impl CharString{
    pub fn new_object(v: Vec<char>) -> Object{
        return Object::String(Rc::new(CharString{data: v}));
    }

    pub fn new_object_str(s: &str) -> Object{
        return Object::String(Rc::new(CharString{data: s.chars().collect()}));
    }

    pub fn new_object_char(c: char) -> Object{
        return Object::String(Rc::new(CharString{data: vec![c]}));
    }
    
    pub fn to_string(&self) -> String {
        return self.data.iter().collect();
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
        self.m.insert(CharString::new_object_str(key),value);
    }

    pub fn insert_fn_plain(&mut self, key: &str, fp: PlainFn,
        argc_min: u32, argc_max: u32
    ) {
        let key = CharString::new_object_str(key);

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
        let t = Table{prototype, map: Map::new()};
        t.map.borrow_mut().insert("value", CharString::new_object_str(s));
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
        let s = CharString::new_object_str(&format!(
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
        let s = CharString::new_object_str(fid);
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
macro_rules! trace_err {
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
    pub fn plain(fp: PlainFn, argc_min: u32, argc_max: u32)
    -> Object
    {
        Object::Function(Rc::new(Function{
            f: EnumFunction::Plain(fp),
            argc: if argc_min==argc_max {argc_min} else {VARIADIC},
            argc_min: argc_min, argc_max: argc_max,
            id: Object::Null
        }))
    }

    pub fn new(f: StandardFn, id: Object, argc_min: u32, argc_max: u32)
    -> Object
    {
        Object::Function(Rc::new(Function{
            f: EnumFunction::Std(f),
            argc: if argc_min==argc_max {argc_min} else {VARIADIC},
            argc_min: argc_min, argc_max: argc_max,
            id: id
        }))
    }

    pub fn mutable(fp: MutableFn, argc_min: u32, argc_max: u32)
    -> Object
    {
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

impl Drop for Table {
    fn drop(&mut self) {
        if let Some(class) = downcast::<Class>(&self.prototype) {
            if class.rte.root_drop.get() {
                let state = &mut class.rte.secondary_state.borrow_mut();
                let env = &mut secondary_env(&class.rte,state);
                let t = Table{
                    prototype: self.prototype.clone(),
                    map: replace(&mut self.map, class.rte.empty_map.clone())
                };
                class.rte.root_drop.set(false);
                class.destructor(t,env);
                loop{
                    let x = class.rte.drop_buffer.borrow_mut().pop();
                    if let Some(mut t) = x {
                        class.destructor(t,env);
                    }else{
                        break;
                    }
                }
                class.rte.root_drop.set(true);
            }else{
                let buffer = &mut class.rte.drop_buffer.borrow_mut();
                buffer.push(Table{
                    prototype: self.prototype.clone(),
                    map: replace(&mut self.map, class.rte.empty_map.clone())
                });
            }
        }
    }
}

pub fn new_module(_id: &str) -> Table{
    Table{prototype: Object::Null, map: Map::new()}
}

pub trait Interface{
    fn as_any(&self) -> &Any;
    fn instance_of_class(&self) -> bool {true}
    fn to_string(&self, _env: &mut Env) -> Result<String,Box<Exception>> {
        Ok("interface object".to_string())
    }
    fn add(&self, _b: &Object, env: &mut Env) -> FnResult {
        Ok(Object::Table(env.rte().unimplemented.clone()))
    }
    fn radd(&self, _a: &Object, env: &mut Env) -> FnResult {
        Ok(Object::Table(env.rte().unimplemented.clone()))
    }
    fn sub(&self, _b: &Object, env: &mut Env) -> FnResult {
        Ok(Object::Table(env.rte().unimplemented.clone()))
    }
    fn rsub(&self, _a: &Object, env: &mut Env) -> FnResult {
        Ok(Object::Table(env.rte().unimplemented.clone()))
    }
    fn mul(&self, _b: &Object, env: &mut Env) -> FnResult {
        Ok(Object::Table(env.rte().unimplemented.clone()))
    }
    fn rmul(&self, _a: &Object, env: &mut Env) -> FnResult {
        Ok(Object::Table(env.rte().unimplemented.clone()))
    }
    fn div(&self, _b: &Object, env: &mut Env) -> FnResult {
        Ok(Object::Table(env.rte().unimplemented.clone()))
    }
    fn rdiv(&self, _a: &Object, env: &mut Env) -> FnResult {
        Ok(Object::Table(env.rte().unimplemented.clone()))
    }
    fn idiv(&self, _b: &Object, env: &mut Env) -> FnResult {
        env.std_exception("Error: a//b is not implemented for objects of this type.")
    }
    fn ridiv(&self, _a: &Object, env: &mut Env) -> FnResult {
        env.std_exception("Error: a//b is not implemented for objects of this type.")
    }
    fn imod(&self, _b: &Object, env: &mut Env) -> FnResult {
        Ok(Object::Table(env.rte().unimplemented.clone()))
    }
    fn rimod(&self, _b: &Object, env: &mut Env) -> FnResult {
        Ok(Object::Table(env.rte().unimplemented.clone()))
    }
    fn pow(&self, _b: &Object, env: &mut Env) -> FnResult {
        env.std_exception("Error: a^b is not implemented for objects of this type.")
    }
    fn rpow(&self, _b: &Object, env: &mut Env) -> FnResult {
        env.std_exception("Error: a^b is not implemented for objects of this type.")
    }

    fn eq_plain(&self, _b: &Object) -> bool {
        false
    }
    fn req_plain(&self, _a: &Object) -> bool {
        false
    }

    fn eq(&self, _b: &Object, env: &mut Env) -> FnResult {
        Ok(Object::Table(env.rte().unimplemented.clone()))
    }
    fn req(&self, _a: &Object, env: &mut Env) -> FnResult {
        Ok(Object::Table(env.rte().unimplemented.clone()))
    }
    fn lt(&self, _b: &Object, env: &mut Env) -> FnResult {
        env.std_exception("Error: a<b is not implemented for objects of this type.")
    }
    fn gt(&self, _b: &Object, env: &mut Env) -> FnResult {
        env.std_exception("Error: a>b is not implemented for objects of this type.")
    }
    fn le(&self, _b: &Object, env: &mut Env) -> FnResult {
        env.std_exception("Error: a<=b is not implemented for objects of this type.")
    }
    fn ge(&self, _b: &Object, env: &mut Env) -> FnResult {
        env.std_exception("Error: a>=b is not implemented for objects of this type.")
    }

    fn rlt(&self, _b: &Object, env: &mut Env) -> FnResult {
        env.std_exception("Error: a<b is not implemented for objects of this type.")
    }
    fn rgt(&self, _b: &Object, env: &mut Env) -> FnResult {
        env.std_exception("Error: a>b is not implemented for objects of this type.")
    }
    fn rle(&self, _b: &Object, env: &mut Env) -> FnResult {
        env.std_exception("Error: a<=b is not implemented for objects of this type.")
    }
    fn rge(&self, _b: &Object, env: &mut Env) -> FnResult {
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
    fn get(&self, _key: &Object, env: &mut Env) -> FnResult {
        env.std_exception("Type error in t.x: getter is not implemented for objects of this type.")
    }
    fn index(&self, _indices: &[Object], env: &mut Env) -> FnResult {
        env.std_exception("Type error in a[i]: indexing is not implemented for objects of this type.")
    }
    fn set_index(&self, _indices: &[Object], _value: &Object, env: &mut Env) -> FnResult {
        env.std_exception("Type error in a[i]=value: indexing is not implemented for objects of this type.")
    }
    fn type_name(&self) -> String {
        "Interface object".to_string()
    }
    fn get_type(&self, env: &mut Env) -> FnResult {
        env.type_error("Type error in type(x): interface object x has no type")
    }
    fn is_instance_of(&self, _type_obj: &Object, _rte: &RTE) -> bool {
        false
    }
    fn hash(&self) -> u64 {
        self as *const _ as *const u8 as usize as u64
    }
    fn iter(&self, env: &mut Env) -> FnResult {
        env.type_error("Type error in iter(x): x is not iterable.")
    }
}

pub fn interface_object_get(
  type_name: &str, key: &Object, env: &mut Env, index: usize
) -> FnResult
{
    let t = &env.rte().interface_types.borrow()[index];
    match t.get(key) {
        Some(value) => return Ok(value),
        None => {
            env.index_error(&format!(
                "Index error in {0}.{1}: {1} not found.", type_name, key
            ))
        }
    }
}

pub fn downcast<T: 'static>(x: &Object) -> Option<&T> {
    if let Object::Interface(ref a) = *x {
        a.as_any().downcast_ref::<T>()
    }else{
        None
    }
}

impl<'a> From<&'a str> for Object {
    fn from(x: &str) -> Object {
        return CharString::new_object_str(x);
    }
}

impl From<char> for Object {
    fn from(x: char) -> Object {
        return CharString::new_object_char(x);
    }
}

impl From<bool> for Object {
    fn from(x: bool) -> Object {
        return Object::Bool(x);
    }
}

impl From<u8> for Object {
    fn from(x: u8) -> Object {
        return Object::Int(x as i32);
    }
}

impl From<u16> for Object {
    fn from(x: u16) -> Object {
        return Object::Int(x as i32);
    }
}

impl From<i32> for Object {
    fn from(x: i32) -> Object {
        return Object::Int(x);
    }
}

impl From<f64> for Object {
    fn from(x: f64) -> Object {
        return Object::Float(x);
    }
}

impl From<Complex64> for Object {
    fn from(x: Complex64) -> Object {
        return Object::Complex(x);
    }
}

impl<T> From<Vec<T>> for Object
where Object: From<T>
{
    fn from(v: Vec<T>) -> Object {
        let mut a: Vec<Object> = Vec::with_capacity(v.len());
        for x in v {
           a.push(Object::from(x));
        }
        return List::new_object(a);
    }
}

pub trait TypeName {
    fn type_name() -> String;
}
impl TypeName for bool {
    fn type_name() -> String {String::from("bool")}
}
impl TypeName for i32 {
    fn type_name() -> String {String::from("i32")}
}
impl TypeName for f64 {
    fn type_name() -> String {String::from("f64")}
}
impl<'a> TypeName for &'a str {
    fn type_name() -> String {String::from("&str")}
}
impl TypeName for String {
    fn type_name() -> String {String::from("String")}
}
impl TypeName for Object {
    fn type_name() -> String {String::from("Object")}
}
impl<T> TypeName for Vec<T>
where T: TypeName
{
    fn type_name() -> String {format!("Vec<{}>",T::type_name())}
}

pub trait Downcast {
    type Output;
    fn try_downcast(x: &Object) -> Option<Self::Output>;
}
impl Downcast for Object {
    type Output = Object;
    fn try_downcast(x: &Object) -> Option<Object> {
        Some(x.clone())
    }
}
impl Downcast for bool {
    type Output = bool;
    fn try_downcast(x: &Object) -> Option<bool> {
        match *x {Object::Bool(x)=>Some(x), _ => None}
    }
}
impl Downcast for i32 {
    type Output = i32;
    fn try_downcast(x: &Object) -> Option<i32> {
        match *x {Object::Int(x)=>Some(x), _ => None}
    }
}
impl Downcast for f64 {
    type Output = f64;
    fn try_downcast(x: &Object) -> Option<f64> {
        match *x {Object::Float(x)=>Some(x), _ => None}
    }
}
impl Downcast for String {
    type Output = String;
    fn try_downcast(x: &Object) -> Option<String> {
        match *x {
            Object::String(ref s) => Some(s.to_string()),
            _ => None
        }
    }
}

impl<T> Downcast for Vec<T>
where T: Downcast<Output=T>
{
    type Output = Vec<T>;
    fn try_downcast(x: &Object) -> Option<Vec<T>> {
        match *x {
            Object::List(ref a) => {
                let a = a.borrow_mut();
                let mut v: Vec<T> = Vec::with_capacity(a.v.len());
                for x in &a.v {
                    match T::try_downcast(x) {
                        Some(x) => v.push(x),
                        None => return None
                    }
                }
                return Some(v);
            },
            _ => None
        }
    }
}


