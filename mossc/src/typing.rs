
#![allow(dead_code)]

use std::rc::Rc;
use std::collections::HashMap;
use std::mem::replace;
use parser::{AST, Symbol, Info};

pub struct Env {
    type_unit: Rc<str>,
    type_bool: Rc<str>,
    type_int: Rc<str>,
    type_string: Rc<str>,
    type_object: Rc<str>,
    type_list: Rc<str>,
    type_tuple: Rc<str>
}

impl Env {
    pub fn new() -> Self {
        Env{
            type_unit: Rc::from("Unit"),
            type_int: Rc::from("Int"),
            type_bool: Rc::from("Bool"),
            type_string: Rc::from("String"),
            type_object: Rc::from("Object"),
            type_list: Rc::from("List"),
            type_tuple: Rc::from("Tuple")
        }
    }
}

pub enum VariableKind {
    Global, Local, Argument(usize)
}

pub struct VariableInfo {
    pub mutable: bool,
    pub kind: VariableKind,
    pub ty: Type
}

impl VariableInfo {
    fn global(ty: Type) -> Self {
        VariableInfo{mutable: false, ty: ty, kind: VariableKind::Global}
    }
}

pub struct SymbolTable {
    pub context: Option<Box<SymbolTable>>,
    pub variables: HashMap<String,VariableInfo>
}

impl SymbolTable {
    pub fn new(env: &Env) -> Self {
        let mut variables: HashMap<String,VariableInfo> = HashMap::new();
        let print_type = Type::Fn(Rc::new(FnType{
            argc_min: 0, argc_max: VARIADIC,
            arg: vec![Type::Atomic(env.type_object.clone())],
            ret: Type::Atomic(env.type_unit.clone())
        }));
        variables.insert("print".into(),VariableInfo::global(print_type));
        SymbolTable{context: None, variables}
    }
    pub fn get(&self, key: &str) -> Option<&VariableInfo> {
        if let Some(value) = self.variables.get(key) {
            return Some(value);
        }else if let Some(ref context) = self.context {
            return context.get(key);
        }else{
            return None;
        }
    }
    pub fn count(&self) -> usize {
        let mut counter = 0;
        for x in self.variables.values() {
            if let VariableKind::Local = x.kind {
                counter+=1;
            }
        }
        return counter;
    }
}

const VARIADIC: usize = ::std::usize::MAX;

pub struct FnType {
    pub argc_min: usize,
    pub argc_max: usize,
    pub arg: Vec<Type>,
    pub ret: Type
}

#[derive(Clone)]
pub enum Type {
    None, Atomic(Rc<str>), App(Rc<Vec<Type>>), Fn(Rc<FnType>)
}

impl Type {
    fn is_app(&self, id: &Rc<str>) -> Option<&[Type]> {
        if let Type::App(a) = self {
            if let Type::Atomic(f) = &a[0] {
                if Rc::ptr_eq(f,id) {
                    return Some(&a[1..]);
                }
            }
        }
        return None;
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Type::Atomic(s) => write!(f,"{}",s),
            Type::App(v) => {
                write!(f,"{}[",v[0])?;
                let mut first = true;
                for x in &v[1..] {
                    if first {
                        first = false;
                        write!(f,"{}",x)?;
                    }else{
                        write!(f,", {}",x)?;
                    }
                }
                write!(f,"]")?;
                return Ok(());
            },
            _ => unimplemented!()
        }
    }
}


#[derive(PartialEq,Eq)]
enum ErrorKind {
    TypeError, UndefinedSymbol
}

pub struct SemanticError {
    line: usize,
    col: usize,
    kind: ErrorKind,
    text: String
}

impl SemanticError {
    pub fn print(&self) {
        println!("Line {}, col {}:",self.line+1,self.col+1);
        match self.kind {
            ErrorKind::TypeError => {
                println!("Type error: {}",self.text);
            },
            ErrorKind::UndefinedSymbol => {
                println!("Undefined variable: {}",self.text);
            }
        }
    }
}

fn type_error(line: usize, col: usize, text: String) -> SemanticError {
    SemanticError{line,col,text,kind: ErrorKind::TypeError}
}

fn undefined_symbol(line: usize, col: usize, text: String) -> SemanticError {
    SemanticError{line,col,text,kind: ErrorKind::UndefinedSymbol}
}

#[derive(PartialEq,Eq)]
enum TypeCmp {
    None, True, False
}

impl From<bool> for TypeCmp {
    fn from(x: bool) -> Self {
        if x {TypeCmp::True} else {TypeCmp::False}
    }
}

fn compare_app_types(p1: &[Type], p2: &[Type]) -> TypeCmp {
    if p1.len() != p2.len() {
        return TypeCmp::False;
    }
    let mut value = TypeCmp::True;
    for i in 0..p1.len() {
        match compare_types(&p1[i],&p2[i]) {
            TypeCmp::None => return TypeCmp::None,
            TypeCmp::False => {value = TypeCmp::False},
            TypeCmp::True => {/*pass*/}
        }
    }
    return value;
}

fn compare_types(t1: &Type, t2: &Type) -> TypeCmp {
    match t1 {
        Type::None => TypeCmp::None,
        Type::Atomic(t1) => {
            match t2 {
                Type::Atomic(t2) => TypeCmp::from(t1==t2),
                Type::None => TypeCmp::None,
                Type::App(_) => TypeCmp::False,
                Type::Fn(_) => panic!()
            }
        },
        Type::App(ref p1) => {
            match t2 {
                Type::None => TypeCmp::None,
                Type::Atomic(_) => TypeCmp::False,
                Type::App(ref p2) => compare_app_types(p1,p2),
                Type::Fn(_) => panic!()
            }
        },
        Type::Fn(_) => {
            panic!();
        }
    }
}

fn is_homogeneous(a: &[Type]) -> bool {
    let x = &a[0];
    for y in &a[1..] {
        match compare_types(x,y) {
            TypeCmp::None => return false,
            TypeCmp::False => return false,
            TypeCmp::True => {}
        }
    }
    return true;
}

fn is_atomic_type(ty: &Type, id: &Rc<str>) -> bool {
    if let Type::Atomic(ty) = ty {
        return Rc::ptr_eq(ty,id);
    }
    return false;
}

fn is_subtype_eq_elementwise(env: &Env, a: &[Type], b: &[Type]) -> bool {
    for i in 0..a.len() {
        if !is_subtype_eq(env,&a[i],&b[i]) {return false;}
    }
    return true;
}

fn is_subtype_eq_object(env: &Env, ty: &Type) -> bool {
    if let Type::Atomic(ty) = ty {
        if Rc::ptr_eq(ty,&env.type_object) ||
           Rc::ptr_eq(ty,&env.type_unit) ||
           Rc::ptr_eq(ty,&env.type_bool) ||
           Rc::ptr_eq(ty,&env.type_int) ||
           Rc::ptr_eq(ty,&env.type_string)
        {
            return true;
        }else{
            return false;
        }
    }else if let Some(a) = ty.is_app(&env.type_tuple) {
        for x in a {
            if !is_subtype_eq_object(env,x) {return false;}
        }
        return true;
    }else if let Some(a) = ty.is_app(&env.type_list) {
        return is_subtype_eq_object(env,&a[0]);
    }else{
        return false;
    }
}

fn is_subtype_eq(env: &Env, t1: &Type, t2: &Type) -> bool {
    match compare_types(t1,t2) {
        TypeCmp::True => return true,
        TypeCmp::False => {},
        TypeCmp::None => {}
    }
    if is_atomic_type(t2,&env.type_object) {
        return is_subtype_eq_object(env,t1);
    }
    if let Some(a) = t1.is_app(&env.type_tuple) {
        if let Some(b) = t2.is_app(&env.type_list) {
            if is_homogeneous(a) {
                return is_subtype_eq(env,&a[0],&b[0]);
            }else{
                for x in a {
                    if !is_subtype_eq(env,x,&b[0]) {return false;}
                }
                return true;
            }
        }else if let Some(b) = t2.is_app(&env.type_tuple) {
            if a.len()==b.len() {
                return is_subtype_eq_elementwise(env,a,b);
            }
        }
    }
    return false;
}

fn type_from_signature(env: &Env, t: &AST) -> Type {
    if t.value == Symbol::None {
        return Type::None;
    }else if let Info::Id(id) = &t.info {
        return match &id[..] {
            "Bool" => Type::Atomic(env.type_bool.clone()),
            "Int" => Type::Atomic(env.type_int.clone()),
            "String" => Type::Atomic(env.type_string.clone()),
            "Object" => Type::Atomic(env.type_object.clone()),
            _ => panic!()
        };
    }else if t.value == Symbol::Application {
        let a = t.argv();
        if let Info::Id(id) = &a[0].info {
            if id=="List" {
                let parameter = type_from_signature(env,&a[1]);
                return Type::App(Rc::new(vec![Type::Atomic(env.type_list.clone()),parameter]));
            }else if id=="Tuple" {
                let mut v: Vec<Type> = Vec::with_capacity(a.len());
                v.push(Type::Atomic(env.type_tuple.clone()));
                for x in &a[1..] {
                    let parameter = type_from_signature(env,x);
                    v.push(parameter);
                }
                return Type::App(Rc::new(v));
            }else{
                panic!();
            }
        }else{
            panic!();
        }
    }else{
        panic!();
    }
}

pub struct TypeChecker {
    pub symbol_table: SymbolTable,
    pub ret_stack: Vec<Type>
}

impl TypeChecker {

pub fn new(env: &Env) -> Self {
    let symbol_table = SymbolTable::new(&env);
    return TypeChecker{symbol_table, ret_stack: Vec::with_capacity(8)};
}

fn type_check_binary_operator(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let ty1 = self.type_check_node(env,&a[0])?;
    let ty2 = self.type_check_node(env,&a[1])?;
    return match compare_types(&ty1,&ty2) {
        TypeCmp::True => Ok(ty1),
        TypeCmp::False => Err(type_error(t.line,t.col,format!(
            "x{}y is not defined for x: {}, y: {}.",t.value,ty1,ty2
        ))),
        TypeCmp::None => unimplemented!()
    };
}

fn type_check_block(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let n = a.len();
    for i in 0..n-1 {
        let _ = self.type_check_node(env,&a[i])?;
    }
    let block_type = self.type_check_node(env,&a[n-1])?;
    return Ok(block_type);
}

fn type_check_let(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let id = match &a[0].info {Info::Id(id) => id.clone(), _ => unreachable!()};
    let ty = type_from_signature(env,&a[1]);
    let ty_expr = self.type_check_node(env,&a[2])?;
    let ty_of_id;
    if let Type::None = ty_expr {
        unimplemented!()
    }else{
        if let Type::None = ty {
            ty_of_id = ty_expr;
        }else{
            match compare_types(&ty,&ty_expr) {
                TypeCmp::True => {ty_of_id = ty;},
                TypeCmp::False => {
                    if is_subtype_eq(env,&ty_expr,&ty) {
                        ty_of_id = ty;
                    }else{
                        return Err(type_error(t.line,t.col,
                            format!("\n    expected {}: {},\n     found type {}.",
                                id,ty,ty_expr)
                        ))
                    }
                },
                TypeCmp::None => {unimplemented!()}
            }
        }
        if self.symbol_table.variables.contains_key(&id) {
            panic!();
        }
        self.symbol_table.variables.insert(id,VariableInfo{
            mutable: false, ty: ty_of_id, kind: VariableKind::Global
        });
    }
    return Ok(Type::Atomic(env.type_unit.clone()));
}

fn type_check_variable(&mut self, t: &AST, id: &String)
-> Result<Type,SemanticError>
{
    if let Some(t) = self.symbol_table.get(id) {
        return Ok(t.ty.clone());
    }else{
        return Err(undefined_symbol(t.line,t.col,format!("{}",id)));
    }
}

fn type_check_tuple(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let mut v: Vec<Type> = Vec::with_capacity(a.len()+1);
    v.push(Type::Atomic(env.type_tuple.clone()));
    for x in a {
        let ty = self.type_check_node(env,x)?;
        v.push(ty);
    }
    return Ok(Type::App(Rc::new(v)));
}

fn type_check_application(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let argv = &a[1..];
    let argc = argv.len();
    let fn_type = self.type_check_node(env,&a[0])?;
    if let Type::Fn(ref sig) = fn_type {
        if argc<sig.argc_min || argc>sig.argc_max {
            let id = match a[0].info {Info::Id(ref s)=>s, _ => panic!()};
            return Err(type_error(t.line,t.col,
                format!("\n  function {} has argument count {}..{},\n  found application of argument count {}.",
                    id,sig.argc_min,sig.argc_max,argc)
            ));
        }
        for i in 0..argc {
            let ty = self.type_check_node(env,&argv[i])?;
            let j = if i<sig.arg.len() {i} else {sig.arg.len()-1};
            if !is_subtype_eq(env,&ty,&sig.arg[j]) {
                return Err(type_error(t.line,t.col,
                    format!("function argument {}\n  expected type {},\n  found type {}.",
                        i,sig.arg[j],ty)
                ));
            }
        }
        return Ok(sig.ret.clone());
    }
    return Err(type_error(t.line,t.col,
        format!("cannot apply a value of type {}.",fn_type)
    ));
}

fn type_check_function(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let header = match t.info {
        Info::FnHeader(ref h) => h,
        _ => unreachable!()
    };

    let mut variables: HashMap<String,VariableInfo> = HashMap::new();

    let ret = type_from_signature(env,&header.ret_type);
    let mut arg: Vec<Type> = Vec::with_capacity(header.argv.len());
    let argv = &header.argv;
    for i in 0..argv.len() {
        let ty = type_from_signature(env,&argv[i].ty);
        arg.push(ty.clone());
        variables.insert(argv[i].id.clone(),VariableInfo{
            mutable: false, ty,
            kind: VariableKind::Argument(i+1)
        });
    }
    let context = replace(&mut self.symbol_table,
        SymbolTable{variables, context: None}
    );
    self.symbol_table.context = Some(Box::new(context));

    let body = &t.argv()[0];
    self.ret_stack.push(ret.clone());
    let value = self.type_check_node(env,body);
    self.ret_stack.pop();
    value?;

    let context = match self.symbol_table.context.take() {
        Some(value) => value, None => unreachable!()
    };

    *header.symbol_table.borrow_mut()
       = Some(replace(&mut self.symbol_table,*context));

    return Ok(Type::Fn(Rc::new(FnType{
        argc_min: header.argv.len(),
        argc_max: header.argv.len(),
        arg, ret
    })));
}

fn type_check_return(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let x = &t.argv()[0];
    let ty_ret = self.type_check_node(env,x)?;
    if let Some(ty) = self.ret_stack.last() {
        if is_subtype_eq(env,&ty_ret,ty) {
            return Ok(Type::Atomic(env.type_unit.clone()));
        }else{
            return Err(type_error(t.line,t.col,format!(
                "expected type: {},\n found type: {}", ty, ty_ret
            )));
        }
    }else{
        panic!();
    }
}

fn type_check_node(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    match t.value {
        Symbol::Item => {
            match t.info {
                Info::Int(_) => return Ok(Type::Atomic(env.type_int.clone())),
                Info::String(_) => return Ok(Type::Atomic(env.type_string.clone())),
                Info::Id(ref id) => return self.type_check_variable(t,id),
                _ => unimplemented!()
            }
        },
        Symbol::False | Symbol::True => {
            return Ok(Type::Atomic(env.type_bool.clone()));
        },
        Symbol::Plus | Symbol::Minus | Symbol::Ast | Symbol::Div |
        Symbol::Pow
        => {
            return self.type_check_binary_operator(env,t);
        },
        Symbol::Block => {
            return self.type_check_block(env,t);
        },
        Symbol::Let => {
            return self.type_check_let(env,t);
        },
        Symbol::List => {
            return self.type_check_tuple(env,t);
        },
        Symbol::Application => {
            return self.type_check_application(env,t);
        },
        Symbol::Function => {
            return self.type_check_function(env,t);
        },
        Symbol::Return => {
            return self.type_check_return(env,t);
        },
        _ => {
            unimplemented!()
        }
    }
}

pub fn type_check(&mut self, env: &Env, t: &AST)
-> Result<(),SemanticError>
{
    let _ = self.type_check_node(env,t)?;
    return Ok(());
}

}


