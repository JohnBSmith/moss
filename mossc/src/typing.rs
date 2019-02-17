
#![allow(dead_code)]

use std::rc::Rc;
use parser::{AST, Symbol, Info};

pub struct Env {
    type_int: Rc<str>,
    type_bool: Rc<str>,
    type_string: Rc<str>
}

pub struct SymbolTable {
    context: Option<Box<SymbolTable>>
}

enum Type {
    None, Atomic(Rc<str>), App(Rc<Vec<Type>>)
}

struct TypeError {
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
-> Result<Type,TypeError>
{
    let a = get_argv(t);
    let ty1 = type_check_node(env,&a[0],symbol_table)?;
    let ty2 = type_check_node(env,&a[1],symbol_table)?;
    return match compare_types(&ty1,&ty2) {
        TypeCmp::True => Ok(ty1),
        TypeCmp::False => Err(TypeError{}),
        TypeCmp::None => unimplemented!()
    };
}

fn type_check_node(env: &Env, t: &AST, symbol_table: &mut SymbolTable)
-> Result<Type,TypeError>
{
    match t.value {
        Symbol::Item => {
            match t.info {
                Info::Int(_) => return Ok(Type::Atomic(env.type_int.clone())),
                Info::String(_) => return Ok(Type::Atomic(env.type_string.clone())),
                _ => unimplemented!()
            }
        },
        Symbol::False | Symbol::True => {
            return Ok(Type::Atomic(env.type_bool.clone()));
        },
        Symbol::Plus | Symbol::Minus | Symbol::Ast | Symbol::Div => {
            return type_check_binary_operator(env,t,symbol_table);
        },
        _ => {
            unimplemented!()
        }
    }
}

pub fn type_check(env: &Env, t: &AST, symbol_table: &mut SymbolTable){
    let _ = type_check_node(env,t,symbol_table);
}
