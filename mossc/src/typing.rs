
#![allow(dead_code)]

use std::rc::Rc;
use std::collections::HashMap;
use parser::{AST, Symbol, Info};

pub struct Env {
    type_unit: Rc<str>,
    type_bool: Rc<str>,
    type_int: Rc<str>,
    type_string: Rc<str>
}

impl Env {
    pub fn new() -> Self {
        Env{
            type_unit: Rc::from("Unit"),
            type_int: Rc::from("Int"),
            type_bool: Rc::from("Bool"),
            type_string: Rc::from("String")
        }
    }
}

pub struct VariableInfo {
    pub mutable: bool,
    pub ty: Type
}

pub struct SymbolTable {
    context: Option<Box<SymbolTable>>,
    variables: HashMap<String,VariableInfo>
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable{context: None, variables: HashMap::new()}
    }
}

#[derive(Clone)]
pub enum Type {
    None, Atomic(Rc<str>), App(Rc<Vec<Type>>)
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

fn get_argv(t: &AST) -> &[Rc<AST>] {
    if let Some(a) = &t.a {
        return a;
    }else{
        unreachable!();
    }
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
                Type::App(_) => TypeCmp::False
            }
        },
        Type::App(ref p1) => {
            match t2 {
                Type::None => TypeCmp::None,
                Type::Atomic(_) => TypeCmp::False,
                Type::App(ref p2) => compare_app_types(p1,p2)
            }
        }
    }
}

fn type_check_binary_operator(env: &Env, t: &AST, symbol_table: &mut SymbolTable)
-> Result<Type,SemanticError>
{
    let a = get_argv(t);
    let ty1 = type_check_node(env,&a[0],symbol_table)?;
    let ty2 = type_check_node(env,&a[1],symbol_table)?;
    return match compare_types(&ty1,&ty2) {
        TypeCmp::True => Ok(ty1),
        TypeCmp::False => Err(type_error(t.line,t.col,String::from("todo"))),
        TypeCmp::None => unimplemented!()
    };
}

fn type_check_block(env: &Env, t: &AST, symbol_table: &mut SymbolTable)
-> Result<Type,SemanticError>
{
    let a = get_argv(t);
    let n = a.len();
    for i in 0..n-1 {
        let _ = type_check_node(env,&a[i],symbol_table)?;
    }
    let block_type = type_check_node(env,&a[n-1],symbol_table)?;
    return Ok(block_type);
}

fn type_from_signature(env: &Env, t: &AST) -> Type {
    if t.value == Symbol::None {
        return Type::None;
    }else if let Info::Id(id) = &t.info {
        match &id[..] {
            "Bool" => return Type::Atomic(env.type_bool.clone()),
            "Int" => return Type::Atomic(env.type_int.clone()),
            "String" => return Type::Atomic(env.type_string.clone()),
            _ => panic!()
        }
    }else if t.value == Symbol::Application {
        unimplemented!();
    }else{
        panic!();
    }
}

fn type_check_let(env: &Env, t: &AST, symbol_table: &mut SymbolTable)
-> Result<Type,SemanticError>
{
    let a = get_argv(t);
    let id = match &a[0].info {Info::Id(id) => id.clone(), _ => unreachable!()};
    let ty = type_from_signature(env,&a[1]);
    let ty_expr = type_check_node(env,&a[2],symbol_table)?;
    if let Type::None = ty_expr {
        unimplemented!()
    }else{
        if let Type::None = ty {
            // pass
        }else{
            match compare_types(&ty,&ty_expr) {
                TypeCmp::True => {},
                TypeCmp::False => return Err(type_error(t.line,t.col,String::from("todo"))),
                TypeCmp::None => {unimplemented!()}
            }
        }
        if symbol_table.variables.contains_key(&id) {
            panic!();
        }
        symbol_table.variables.insert(id,VariableInfo{
            mutable: false, ty: ty_expr
        });
    }
    return Ok(Type::Atomic(env.type_unit.clone()));
}

fn type_check_variable(t: &AST, id: &String, symbol_table: &mut SymbolTable)
-> Result<Type,SemanticError>
{
    if let Some(t) = symbol_table.variables.get(id) {
        return Ok(t.ty.clone());
    }else{
        return Err(undefined_symbol(t.line,t.col,format!("{}",id)));
    }
}

fn type_check_node(env: &Env, t: &AST, symbol_table: &mut SymbolTable)
-> Result<Type,SemanticError>
{
    match t.value {
        Symbol::Item => {
            match t.info {
                Info::Int(_) => return Ok(Type::Atomic(env.type_int.clone())),
                Info::String(_) => return Ok(Type::Atomic(env.type_string.clone())),
                Info::Id(ref id) => return type_check_variable(t,id,symbol_table),
                _ => unimplemented!()
            }
        },
        Symbol::False | Symbol::True => {
            return Ok(Type::Atomic(env.type_bool.clone()));
        },
        Symbol::Plus | Symbol::Minus | Symbol::Ast | Symbol::Div => {
            return type_check_binary_operator(env,t,symbol_table);
        },
        Symbol::Block => {
            return type_check_block(env,t,symbol_table);
        },
        Symbol::Let => {
            return type_check_let(env,t,symbol_table);
        },
        _ => {
            unimplemented!()
        }
    }
}

pub fn type_check(env: &Env, t: &AST, symbol_table: &mut SymbolTable)
-> Result<(),SemanticError>
{
    let _ = type_check_node(env,t,symbol_table)?;
    return Ok(());
}
