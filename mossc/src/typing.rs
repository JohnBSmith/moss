
#![allow(dead_code)]

use std::rc::Rc;
use std::collections::HashMap;
use parser::{AST, Symbol, Info};

type Env = Rc<Environment>;
struct Environment {
    env: Option<Env>,
    map: HashMap<String,Rc<str>>
}
impl Environment {
    fn from_sig(env: &Env, sig: &[TypeVariable]) -> Env {
        let mut m: HashMap<String,Rc<str>> = HashMap::new();
        for tv in sig {
            m.insert((&*tv.id).into(),tv.id.clone());
        }
        return Rc::new(Environment{env: Some(env.clone()), map: m});
    }
    fn get(&self, id: &String) -> Option<Rc<str>> {
        if let Some(value) = self.map.get(id) {
            return Some(value.clone());
        }else if let Some(env) = &self.env {
            return env.get(id);
        }else{
            return None;
        }
    }
    fn contains(&self, ty0: &Rc<str>) -> bool {
        for ty in self.map.values() {
            if Rc::ptr_eq(ty0,ty) {return true;}
        }
        if let Some(env) = &self.env {
            return env.contains(ty0);
        }else{
            return false;
        }
    }
}

pub struct TypeTable {
    type_unit: Rc<str>,
    type_bool: Rc<str>,
    type_int: Rc<str>,
    type_string: Rc<str>,
    type_object: Rc<str>,
    type_list: Rc<str>,
    type_tuple: Rc<str>,
    type_range: Rc<str>
}

impl TypeTable {
    pub fn new() -> Rc<Self> {
        Rc::new(TypeTable {
            type_unit: Rc::from("Unit"),
            type_int: Rc::from("Int"),
            type_bool: Rc::from("Bool"),
            type_string: Rc::from("String"),
            type_object: Rc::from("Object"),
            type_list: Rc::from("List"),
            type_tuple: Rc::from("Tuple"),
            type_range: Rc::from("Range")
        })
    }
}

#[derive(Debug)]
pub enum VariableKind {
    Global, Local(usize), Argument(usize), FnSelf
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

pub struct SymbolTableNode {
    pub context: Option<usize>,
    pub variables: HashMap<String,VariableInfo>,
    pub local_count_max: usize,
    pub local_count: usize
}

pub struct SymbolTable {
    pub list: Vec<SymbolTableNode>,
    pub index: usize
}

impl SymbolTable {
    pub fn new(tab: &TypeTable) -> Self {
        let mut variables: HashMap<String,VariableInfo> = HashMap::new();
        let type_of_print = Type::Fn(Rc::new(FnType{
            argc_min: 0, argc_max: VARIADIC,
            arg: vec![Type::Atomic(tab.type_object.clone())],
            ret: Type::Atomic(tab.type_unit.clone())
        }));
        variables.insert("print".into(),VariableInfo::global(type_of_print));

        let type_var: Rc<str> = Rc::from("T");
        let type_of_len = Type::Fn(Rc::new(FnType{
            argc_min: 1, argc_max: 1,
            arg: vec![Type::App(Rc::new(vec![
                Type::Atomic(tab.type_list.clone()),
                Type::Atomic(type_var.clone())
            ]))],
            ret: Type::Atomic(tab.type_int.clone())
        }));
        let type_of_len = Type::poly1(type_var,type_of_len);
        variables.insert("len".into(),VariableInfo::global(type_of_len));

        let type_var: Rc<str> = Rc::from("T");
        let type_of_str = Type::Fn(Rc::new(FnType{
            argc_min: 1, argc_max: 1,
            arg: vec![Type::Atomic(type_var.clone())],
            ret: Type::Atomic(tab.type_string.clone())
        }));
        let type_of_str = Type::poly1(type_var,type_of_str);
        variables.insert("str".into(),VariableInfo::global(type_of_str));
        
        let type_var: Rc<str> = Rc::from("T");
        let type_of_list_push = Type::Fn(Rc::new(FnType{
            argc_min: 2, argc_max: 2,
            arg: vec![
                Type::App(Rc::new(vec![
                    Type::Atomic(tab.type_list.clone()),
                    Type::Atomic(type_var.clone())
                ])),
                Type::Atomic(type_var.clone())
            ],
            ret: Type::Atomic(tab.type_unit.clone())
        }));
        let type_of_list_push = Type::poly1(type_var,type_of_list_push);
        variables.insert("list_push".into(),VariableInfo::global(type_of_list_push));

        let type_of_list = Type::Fn(Rc::new(FnType{
            argc_min: 1, argc_max: 1,
            arg: vec![Type::App(Rc::new(vec![
                Type::Atomic(tab.type_range.clone()),
                Type::Atomic(tab.type_int.clone()),
                Type::Atomic(tab.type_int.clone()),
                Type::Atomic(tab.type_unit.clone())
            ]))],
            ret: Type::App(Rc::new(vec![
                Type::Atomic(tab.type_list.clone()),
                Type::Atomic(tab.type_int.clone())
            ]))
        }));
        variables.insert("list".into(),VariableInfo::global(type_of_list));

        let type_of_iter = Type::Fn(Rc::new(FnType{
            argc_min: 0, argc_max: VARIADIC,
            arg: vec![Type::Atomic(tab.type_object.clone())],
            ret: Type::Atomic(tab.type_object.clone())
        }));
        variables.insert("iter".into(),VariableInfo::global(type_of_iter));

        let node = SymbolTableNode{
            context: None, variables,
            local_count: 0, local_count_max: 0
        };
        let table = SymbolTable{
            index: 0, list: vec![node]
        };
        return table;
    }
    pub fn get(&self, key: &str) -> Option<&VariableInfo> {
        let mut node = &self.list[self.index];
        loop{
            if let Some(value) = node.variables.get(key) {
                return Some(value);
            }else if let Some(context) = node.context {
                node = &self.list[context];
            }else{
                return None;
            }
        }
    }
    pub fn local_count_max(&self) -> usize {
        let index = self.index;
        return self.list[index].local_count_max;
    }
    fn print(&self){
        println!("index: {}",self.index);
        for (i,x) in self.list.iter().enumerate() {
            print!("[{}] ",i);
            for id in x.variables.keys() {
                print!("{}, ",id);
            }
            println!();
        }
        println!();
    }

    pub fn variable_binding(&mut self, global: bool, mutable: bool,
        id: String, typ: Type
    ) {
        let index = self.index;
        let node = &mut self.list[index];
        if node.variables.contains_key(&id) {
            panic!();
        }
        let kind = if global {
            VariableKind::Global
        }else{
            node.local_count+=1;
            if node.local_count > node.local_count_max {
                node.local_count_max = node.local_count;
            }
            VariableKind::Local(node.local_count-1)
        };
        node.variables.insert(id,VariableInfo{
            mutable, ty: typ, kind
        });
    }
}

const VARIADIC: usize = ::std::usize::MAX;

pub struct FnType {
    pub argc_min: usize,
    pub argc_max: usize,
    pub arg: Vec<Type>,
    pub ret: Type
}

pub struct TypeVariable {
    pub id: Rc<str>,
    pub class: Type
}

pub struct PolyType {
    pub variables: Vec<TypeVariable>,
    pub scheme: Type
}

#[derive(Clone)]
pub enum Type {
    None, Atomic(Rc<str>), App(Rc<Vec<Type>>), Fn(Rc<FnType>),
    Poly(Rc<PolyType>)
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
    fn list(env: &TypeTable, el: Type) -> Type {
        let list_type = Type::Atomic(env.type_list.clone());
        return Type::App(Rc::new(vec![list_type,el]));
    }
    fn poly1(type_var: Rc<str>, scheme: Type) -> Type {
        Type::Poly(Rc::new(PolyType{
            variables: vec![TypeVariable{id: type_var.clone(), class: Type::None}],
            scheme
        }))
    }
    fn is_atomic(&self, typ_id: &Rc<str>) -> bool {
        if let Type::Atomic(typ) = self {
            Rc::ptr_eq(typ,typ_id)
        }else{
            false
        }
    }
}

impl std::fmt::Display for PolyType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"forall[")?;
        let mut first = true;
        for tv in &self.variables {
            if first {first = false;} else {write!(f,", ")?;}
            write!(f,"{}",tv.id)?;
        }
        write!(f,"] ")?;
        write!(f,"{}",self.scheme)
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
            Type::Fn(t) => {
                if t.arg.len()==1 {
                    write!(f,"{}=>{}",&t.arg[0],&t.ret)
                }else{
                    unimplemented!();
                }
            },
            Type::Poly(p) => {
                write!(f,"{}",p)
            },
            Type::None => {
                write!(f,"_")
            }
        }
    }
}

#[derive(PartialEq,Eq)]
enum ErrorKind {
    Error, TypeError, UndefinedSymbol
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
            ErrorKind::Error => {
                println!("Error: {}",self.text);
            },
            ErrorKind::TypeError => {
                println!("Type error: {}",self.text);
            },
            ErrorKind::UndefinedSymbol => {
                println!("Undefined variable: {}",self.text);
            }
        }
    }
}

fn error(line: usize, col: usize, text: String) -> SemanticError {
    SemanticError{line,col,text,kind: ErrorKind::Error}
}

fn type_error(line: usize, col: usize, text: String) -> SemanticError {
    SemanticError{line,col,text,kind: ErrorKind::TypeError}
}

fn undefined_symbol(line: usize, col: usize, text: String) -> SemanticError {
    SemanticError{line,col,text,kind: ErrorKind::UndefinedSymbol}
}

fn type_mismatch(expected: &Type, given: &Type) -> String {
    format!("\n  expected type {}  \n  found    type {}",expected,given)
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

fn compare_fn_types(f1: &FnType, f2: &FnType) -> TypeCmp {
    match compare_types(&f1.ret,&f2.ret) {
        TypeCmp::None => return TypeCmp::None,
        TypeCmp::False => return TypeCmp::False,
        TypeCmp::True => {/*pass*/}
    }
    if f1.arg.len() != f2.arg.len() ||
       f1.argc_min != f2.argc_min ||
       f1.argc_max != f2.argc_max
    {
        return TypeCmp::False;
    }
    for i in 0..f1.arg.len() {
        match compare_types(&f1.arg[i],&f2.arg[i]) {
            TypeCmp::None => return TypeCmp::None,
            TypeCmp::False => return TypeCmp::False,
            TypeCmp::True => {/*pass*/}
        }        
    }
    return TypeCmp::True;
}

fn compare_types(t1: &Type, t2: &Type) -> TypeCmp {
    match t1 {
        Type::None => TypeCmp::None,
        Type::Atomic(t1) => {
            match t2 {
                Type::Atomic(t2) => TypeCmp::from(t1==t2),
                Type::None => TypeCmp::None,
                Type::App(_) => TypeCmp::False,
                Type::Fn(_) => TypeCmp::False,
                _ => panic!()
            }
        },
        Type::App(ref p1) => {
            match t2 {
                Type::None => TypeCmp::None,
                Type::Atomic(_) => TypeCmp::False,
                Type::App(ref p2) => compare_app_types(p1,p2),
                Type::Fn(_) => TypeCmp::False,
                _ => panic!()
            }
        },
        Type::Fn(ref f1) => {
            if let Type::Fn(ref f2) = t2 {
                compare_fn_types(f1,f2)
            }else{
                TypeCmp::False
            }
        },
        _ => panic!()
    }
}

fn is_homogeneous(a: &[Type]) -> bool {
    if a.len()==0 {return true;}
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

fn is_subtype_eq_elementwise(env: &TypeTable, a: &[Type], b: &[Type])
-> bool {
    for i in 0..a.len() {
        if !is_subtype_eq(env,&a[i],&b[i]) {return false;}
    }
    return true;
}

fn is_subtype_eq_object(env: &TypeTable, ty: &Type) -> bool {
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
    }else if let Some(a) = ty.is_app(&env.type_range) {
        for x in a {
            if !is_subtype_eq_object(env,x) {return false;}
        }
        return true;
    }else{
        return false;
    }
}

fn is_subtype_eq(env: &TypeTable, t1: &Type, t2: &Type) -> bool {
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
                if a.len()==0 {
                    return true;
                }else{
                    return is_subtype_eq(env,&a[0],&b[0]);
                }
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
    }else if let Some(a) = t1.is_app(&env.type_list) {
        if let Some(b) = t2.is_app(&env.type_list){
            return is_subtype_eq(env,&a[0],&b[0]);
        }
    }
    return false;
}

struct Substitution {
    map: HashMap<Rc<str>,Type>
}
impl Substitution {
    fn new() -> Self {
        Self{map: HashMap::new()}
    }
    fn apply(&self, typ: &Type) -> Type {
        match typ {
            Type::Atomic(typ) => {
                return match self.map.get(typ) {
                    Some(subs) => subs.clone(),
                    None => Type::Atomic(typ.clone())
                };
            },
            Type::App(app) => {
                Type::App(Rc::new(app.iter().map(|x|
                    self.apply(x)).collect::<Vec<Type>>()))
            },
            _ => {
                todo!();
            }
        }
    }
}

pub struct TypeChecker {
    pub symbol_table: SymbolTable,
    pub ret_stack: Vec<Type>,
    pub global_context: bool,
    pub tab: Rc<TypeTable>
}

impl TypeChecker {

pub fn new(tab: &Rc<TypeTable>) -> Self {
    let symbol_table = SymbolTable::new(&tab);
    return TypeChecker{
        symbol_table,
        ret_stack: Vec::with_capacity(8),
        global_context: true,
        tab: tab.clone()
    };
}

fn type_from_signature(&self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    if t.value == Symbol::None {
        return Ok(Type::None);
    }else if let Info::Id(id) = &t.info {
        if let Some(ty) = env.get(id) {
            return Ok(Type::Atomic(ty));
        }
        return Ok(match &id[..] {
            "Unit" => Type::Atomic(self.tab.type_unit.clone()),
            "Bool" => Type::Atomic(self.tab.type_bool.clone()),
            "Int" => Type::Atomic(self.tab.type_int.clone()),
            "String" => Type::Atomic(self.tab.type_string.clone()),
            "Object" => Type::Atomic(self.tab.type_object.clone()),
            id => {
                return Err(type_error(t.line,t.col,format!(
                    "Unknown type: {}.", id
                )));
            }
        });
    }else if t.value == Symbol::Application {
        let a = t.argv();
        if let Info::Id(id) = &a[0].info {
            if id=="List" {
                let parameter = self.type_from_signature(env,&a[1])?;
                return Ok(Type::App(Rc::new(vec![Type::Atomic(self.tab.type_list.clone()),parameter])));
            }else if id=="Tuple" {
                let mut v: Vec<Type> = Vec::with_capacity(a.len());
                v.push(Type::Atomic(self.tab.type_tuple.clone()));
                for x in &a[1..] {
                    let parameter = self.type_from_signature(env,x)?;
                    v.push(parameter);
                }
                return Ok(Type::App(Rc::new(v)));
            }else{
                panic!();
            }
        }else{
            panic!();
        }
    }else if t.value == Symbol::Fn {
        let a = t.argv();
        let n = a.len();
        let mut arg: Vec<Type> = Vec::with_capacity(n);
        for x in &a[..n-1] {
            let ty = self.type_from_signature(env,x)?;
            arg.push(ty);
        }
        let ret = self.type_from_signature(env,&a[n-1])?;
        return Ok(Type::Fn(Rc::new(FnType {
            argc_min: n-1, argc_max: n-1,
            arg, ret
        })));
    }else if t.value == Symbol::Unit {
        return Ok(Type::Atomic(self.tab.type_unit.clone()));
    }else{
        unimplemented!("{}",t.value);
    }
}

fn type_check_binary_operator(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let ty1 = self.type_check_node(env,&a[0])?;
    let ty2 = self.type_check_node(env,&a[1])?;
    return if is_subtype_eq(&self.tab,&ty2,&ty1) {
        Ok(ty1)
    }else if is_subtype_eq(&self.tab,&ty1,&ty2) {
        Ok(ty2)
    }else{
        Err(type_error(t.line,t.col,format!(
            "x{}y is not defined for x: {}, y: {}.",t.value,ty1,ty2
        )))    
    };
}

fn type_check_range(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let ta = self.type_check_node(env,&a[0])?;
    let tb = self.type_check_node(env,&a[1])?;
    let td = self.type_check_node(env,&a[2])?;
    let range = Type::Atomic(self.tab.type_range.clone());
    return Ok(Type::App(Rc::new(vec![range,ta,tb,td])));
}

fn index_homogeneous(&mut self, t: &AST, ty_index: &Type, ty: Type)
-> Result<Type,SemanticError>
{
    let tab = &self.tab;
    if let Some(a) = ty_index.is_app(&tab.type_range) {
        if is_atomic_type(&a[2],&tab.type_unit) {
            if is_atomic_type(&a[0],&tab.type_int) ||
               is_atomic_type(&a[0],&tab.type_unit)
            {
                if is_atomic_type(&a[1],&tab.type_int) ||
                   is_atomic_type(&a[1],&tab.type_unit)
                {
                    return Ok(Type::list(tab,ty));
                }
            }
        }
    }else if is_atomic_type(&ty_index,&tab.type_int) {
        return Ok(ty);
    }
    return Err(type_error(t.line,t.col,format!(
        "a[i] is not defined for i: {}.", ty_index
    )));
}

fn type_check_operator_index(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    if a.len()>2 {
        return Err(type_error(t.line,t.col,String::from(
            "in a[...]: expected only one index."
        )));
    }

    let ty_seq = self.type_check_node(env,&a[0])?;
    let ty_index = self.type_check_node(env,&a[1])?;
    if let Some(a) = ty_seq.is_app(&self.tab.type_list) {
        return self.index_homogeneous(t,&ty_index,a[0].clone());
    }else if let Some(a) = ty_seq.is_app(&self.tab.type_tuple) {
        if is_homogeneous(a) {
            return self.index_homogeneous(t,&ty_index,a[0].clone());
        }
    }
    return Err(type_error(t.line,t.col,format!(
        "expected a in a[i] of indexable type,\n  found type {}.", ty_seq
    )));
}

fn type_check_comparison(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let type1 = self.type_check_node(env,&a[0])?;
    let type2 = self.type_check_node(env,&a[1])?;
    return match compare_types(&type1,&type2) {
        TypeCmp::True => Ok(Type::Atomic(self.tab.type_bool.clone())),
        TypeCmp::False => Err(type_error(t.line,t.col,format!(
            "x{}y is not defined for x: {}, y: {}.",t.value,type1,type2
        ))),
        TypeCmp::None => unimplemented!()
    };
}

fn type_check_eq(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let type1 = self.type_check_node(env,&a[0])?;
    let type2 = self.type_check_node(env,&a[1])?;
    return if is_subtype_eq(&self.tab,&type2,&type1) {
        Ok(Type::Atomic(self.tab.type_bool.clone()))
    }else{
        Err(type_error(t.line,t.col,format!(
            "x{}y is not defined for x: {}, y: {}.",
            t.value, type1, type2
        )))    
    };
}

fn type_check_logical_operator(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let type1 = self.type_check_node(env,&a[0])?;
    let type2 = self.type_check_node(env,&a[1])?;
    if !is_atomic_type(&type1,&self.tab.type_bool) ||
       !is_atomic_type(&type2,&self.tab.type_bool)
    {
        return Err(type_error(t.line,t.col,format!(
            "x {} y is not defined for x: {}, y: {}.",
            t.value, type1, type2
        )));
    }    
    return Ok(Type::Atomic(self.tab.type_bool.clone()));
}

fn type_check_if_expression(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let type0 = self.type_check_node(env,&a[0])?;
    let type1 = self.type_check_node(env,&a[1])?;
    let type2 = self.type_check_node(env,&a[2])?;
    if !is_atomic_type(&type0,&self.tab.type_bool) {
        return Err(type_error(t.line,t.col,format!(
            "expected condition of type bool,\n  found type: {}.",
            type0
        )));
    }
    return if is_subtype_eq(&self.tab,&type2,&type1) {
        Ok(type1)
    }else if is_subtype_eq(&self.tab,&type1,&type2) {
        Ok(type2)
    }else{
        Err(type_error(t.line,t.col,format!(
            "x if c else y expected x and y of the same type,\n  found x: {}, y: {}.",
            type1, type2
        )))
    };
}

fn type_check_block(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let n = a.len();
    if n==0 {
        return Ok(Type::Atomic(self.tab.type_unit.clone()));
    }
    for i in 0..n-1 {
        let _ = self.type_check_node(env,&a[i])?;
    }
    let block_type = self.type_check_node(env,&a[n-1])?;
    return Ok(block_type);
}

fn type_check_let(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let mutable = match t.info {Info::Mut => true, _ => false};
    let a = t.argv();
    let id = match &a[0].info {Info::Id(id) => id.clone(), _ => unreachable!()};
    let ty = self.type_from_signature(env,&a[1])?;
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
                    if is_subtype_eq(&self.tab,&ty_expr,&ty) {
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
        let global = self.global_context;
        self.symbol_table.variable_binding(global,mutable,id,ty_of_id);
    }
    return Ok(Type::Atomic(self.tab.type_unit.clone()));
}

fn type_check_assignment(&mut self, env: &Env, t: &AST)
-> Result<(),SemanticError>
{
    let a = t.argv();
    let id = match &a[0].info {Info::Id(id) => id.clone(), _ => unreachable!()};
    let ty_expr = self.type_check_node(env,&a[1])?;

    let index = self.symbol_table.index;
    let node = &mut self.symbol_table.list[index];
    if let Some(variable_info) = node.variables.get(&id) {
        if !variable_info.mutable {
            return Err(error(t.line,t.col,
                format!("variable '{}' is immutable.",id)
            ));
        }
        let ty = &variable_info.ty;
        if !is_subtype_eq(&self.tab,&ty_expr,&ty) {
            return Err(type_error(t.line,t.col,
                format!("\n    expected {}: {},\n     found type {}.",
                    id,ty,ty_expr)
            ));
        }
    }else{
        return Err(undefined_symbol(t.line,t.col,id));
    }
    return Ok(());
}

fn type_check_variable(&mut self, t: &AST, id: &String)
-> Result<Type,SemanticError>
{
    // self.symbol_table.print();
    if let Some(t) = self.symbol_table.get(id) {
        // println!("{}, kind: {:?}",id,t.kind);
        return Ok(t.ty.clone());
    }else{
        return Err(undefined_symbol(t.line,t.col,format!("{}",id)));
    }
}

fn type_check_tuple(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    if a.is_empty() {
        return Ok(Type::App(Rc::new(vec![
            Type::Atomic(self.tab.type_list.clone()),
            Type::None
        ])));
    }
    let ty0 = self.type_check_node(env,&a[0])?;
    for x in &a[1..] {
        let ty = self.type_check_node(env,x)?;
        match compare_types(&ty0,&ty) {
            TypeCmp::True => {},
            _ => {
                return Err(type_error(t.line,t.col,
                    type_mismatch(&ty0,&ty)
                ));
            }
        }
    }
    return Ok(Type::App(Rc::new(vec![
        Type::Atomic(self.tab.type_list.clone()),
        ty0
    ])));
}

fn unify_fn(&mut self, env: &Env, f1: &FnType, f2: &FnType,
    subs: &mut Substitution
) -> Result<(),String> {
    self.unify(env,&f1.ret,&f2.ret,subs)?;
    if f1.arg.len() != f2.arg.len() ||
       f1.argc_min != f2.argc_min ||
       f1.argc_max != f2.argc_max
    {
        return Err(format!("mismatch in number of arguments:\n  expected {}\n  found {}",
            f1.arg.len(), f2.arg.len()
        ));
    }
    for i in 0..f1.arg.len() {
        self.unify(env,&f1.arg[i],&f2.arg[i],subs)?;
    }
    return Ok(());
}

fn unify(&mut self, env: &Env, expected: &Type, given: &Type,
    subs: &mut Substitution
) -> Result<(),String>
{
    match expected {
        Type::Atomic(expected) => {
            if let Type::Atomic(given) = given {
                if Rc::ptr_eq(expected,given) {return Ok(());}
            }
            if let Some(expected) = subs.map.get(expected) {
                return self.unify(env,&expected.clone(),given,subs);
            }
            if env.contains(expected) {
                subs.map.insert(expected.clone(),given.clone());
                return Ok(());
            }
        },
        Type::App(app_expected) => {
            if let Type::App(app_given) = given {
                if app_expected.len() != app_given.len() {
                    return Err(format!("Mismatch in number of type arguments: \n  expected {}, \n  found {}",
                        expected,given));
                }
                for i in 0..app_expected.len() {
                    self.unify(env,&app_expected[i],&app_given[i],subs)?;
                }
                return Ok(());
            }else{
                return Err(type_mismatch(expected,given));
            }
        },
        Type::Fn(fn_expected) => {
            if let Type::Fn(fn_given) = given {
                return self.unify_fn(env,fn_expected,fn_given,subs);
            }else{
                return Err(type_mismatch(expected,given));
            }
        },
        expected => {
            return Err(format!("Cannot unify {}",expected));
        }
    }
    if is_subtype_eq(&self.tab,given,expected) {
        return Ok(());
    }else{
        return Err(type_mismatch(expected,given));
    }
}

fn type_check_application(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let argv = &a[1..];
    let argc = argv.len();
    let mut subs = Substitution::new();
    let fn_type = self.type_check_node(env,&a[0])?;
    let (sig,poly,env) = match &fn_type {
        Type::Poly(poly) => {
            let env = Environment::from_sig(env,&poly.variables);
            (&poly.scheme,Some(&poly.variables),env)
        },
        ty => (ty,None,env.clone())
    };
    if let Type::Fn(ref sig) = sig {
        if argc<sig.argc_min || argc>sig.argc_max {
            let id = match a[0].info {Info::Id(ref s)=>s, _ => panic!()};
            return Err(type_error(t.line,t.col,
                format!("\n  function {} has argument count {}..{},\n  found application of argument count {}.",
                    id,sig.argc_min,sig.argc_max,argc)
            ));
        }
        for i in 0..argc {
            let ty = self.type_check_node(&env,&argv[i])?;
            let j = if i<sig.arg.len() {i} else {sig.arg.len()-1};
            match self.unify(&env,&sig.arg[j],&ty,&mut subs) {
                Ok(()) => {},
                Err(text) => {
                    return Err(type_error(t.line,t.col,
                        format!("Function argument {}: {}.",i,text)
                    ));
                }
            }
        }
        if poly.is_some() {
            return Ok(subs.apply(&sig.ret));
        }else{
            return Ok(sig.ret.clone());
        }
    }else if sig.is_atomic(&self.tab.type_object) {
        for i in 0..argc {
            let _ty = self.type_check_node(&env,&argv[i])?;        
        }
        return Ok(Type::Atomic(self.tab.type_object.clone()));
    };
    return Err(type_error(t.line,t.col,
        format!("cannot apply a value of type {}.",fn_type)
    ));
}

fn poly_sig(&self, type_variables: &Rc<AST>) -> Vec<TypeVariable> {
    let mut v: Vec<TypeVariable> = Vec::new();
    let a = type_variables.argv();
    for x in a {
        let id: Rc<str> = match &x.info {
            Info::Id(id) => Rc::from(&id[..]),
            _ => unreachable!()
        };
        v.push(TypeVariable{id, class: Type::None});
    }
    return v;
} 

fn type_check_function(&mut self, env_rec: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let header = match t.info {
        Info::FnHeader(ref h) => h,
        _ => unreachable!()
    };

    let mut variables: HashMap<String,VariableInfo> = HashMap::new();

    let (env,sig) = if let Some(type_variables) = &header.type_variables {
        let sig = self.poly_sig(type_variables);
        let env = Environment::from_sig(env_rec,&sig);
        (env,Some(sig))
    }else{
        (env_rec.clone(), None)
    };

    let ret = self.type_from_signature(&env,&header.ret_type)?;
    let mut arg: Vec<Type> = Vec::with_capacity(header.argv.len());
    let argv = &header.argv;
    for i in 0..argv.len() {
        let ty = self.type_from_signature(&env,&argv[i].ty)?;
        arg.push(ty.clone());
        variables.insert(argv[i].id.clone(),VariableInfo{
            mutable: false, ty,
            kind: VariableKind::Argument(i+1)
        });
    }

    let mut ftype = Type::Fn(Rc::new(FnType{
        argc_min: header.argv.len(),
        argc_max: header.argv.len(),
        arg, ret: ret.clone()
    }));
    if let Type::None = ret {
        // pass
    }else{
        if let Some(ref id) = header.id {
            variables.insert(id.clone(),VariableInfo{
                mutable: false, ty: ftype.clone(),
                kind: VariableKind::FnSelf
            });
        }
    };

    let context = self.symbol_table.index;
    self.symbol_table.index = self.symbol_table.list.len();
    self.symbol_table.list.push(SymbolTableNode{
        variables,
        context: Some(context),
        local_count: 0, local_count_max: 0
    });

    let body = &t.argv()[0];
    self.ret_stack.push(ret.clone());
    let global_context = self.global_context;
    self.global_context = false;
    let value = self.type_check_node(&env,body);
    self.global_context = global_context;
    self.ret_stack.pop();
    let ret_type = value?;
    if let Type::None = ret {
        if let Type::Fn(ftype) = &mut ftype {
            if let Some(ftype) = Rc::get_mut(ftype) {
                ftype.ret = ret_type;
            }else{
                unreachable!();
            }
        }else{
            unreachable!();
        }
    }else{
        if !ret_type.is_atomic(&self.tab.type_unit) {
            match compare_types(&ret,&ret_type) {
                TypeCmp::True => {},
                _ => return Err(type_error(t.line,t.col,type_mismatch(&ret,&ret_type)))
            }
        }
    }

    header.symbol_table_index.set(self.symbol_table.index);
    self.symbol_table.index = context;

    if let Some(sig) = sig {
        return Ok(Type::Poly(Rc::new(PolyType{
            variables: sig, scheme: ftype
        })));
    }else{
        return Ok(ftype);
    }
}

fn type_check_return(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let x = &t.argv()[0];
    let ty_ret = self.type_check_node(env,x)?;
    if let Some(ty) = self.ret_stack.last() {
        if is_subtype_eq(&self.tab,&ty_ret,ty) {
            return Ok(Type::Atomic(self.tab.type_unit.clone()));
        }else{
            return Err(type_error(t.line,t.col,type_mismatch(ty,&ty_ret)));
        }
    }else{
        panic!();
    }
}

fn type_check_if_statement(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let n = a.len();
    let mut i = 0;
    while i+1<n {
        let ty_cond = self.type_check_node(env,&a[i])?;
        if !is_atomic_type(&ty_cond,&self.tab.type_bool) {
            return Err(type_error(t.line,t.col,format!(
                "expected condition of type bool,\n  found type: {}.",
                ty_cond
            )));
        }
        self.type_check_node(env,&a[i+1])?;
        i+=2;
    }
    if n%2 != 0 {
        self.type_check_node(env,&a[n-1])?;
    }
    return Ok(Type::Atomic(self.tab.type_unit.clone()));
}

fn type_check_while_statement(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let ty_cond = self.type_check_node(env,&a[0])?;
    if !is_atomic_type(&ty_cond,&self.tab.type_bool) {
        return Err(type_error(t.line,t.col,format!(
            "expected condition of type bool,\n  found type: {}.",
            ty_cond
        )));
    }
    self.type_check_node(env,&a[1])?;
    return Ok(Type::Atomic(self.tab.type_unit.clone()));
}

fn iter_element(&self, iterable: &Type) -> Type {
    if let Some(a) = iterable.is_app(&self.tab.type_list) {
        return a[0].clone();
    }else if let Some(a) = iterable.is_app(&self.tab.type_range) {
        if a[0].is_atomic(&self.tab.type_int) &&
           a[1].is_atomic(&self.tab.type_int) &&
           a[2].is_atomic(&self.tab.type_unit)
        {
            return a[0].clone();
        }else{
            todo!();
        }
    }else{
        todo!();
    }
}

fn type_check_for_statement(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let ty_range = self.type_check_node(env,&a[1])?;
    let typ = self.iter_element(&ty_range);
    let id = match &a[0].info {Info::Id(id) => id.clone(), _ => panic!()};
    let global = self.global_context;
    self.symbol_table.variable_binding(global,false,id,typ);
    self.type_check_node(env,&a[2])?;
    return Ok(Type::Atomic(self.tab.type_unit.clone()));
}

fn type_check_dot(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let typ = self.type_check_node(env,&a[0])?;
    if typ.is_atomic(&self.tab.type_object) {
        return Ok(Type::Atomic(self.tab.type_object.clone()));
    }else{
        let slot = match a[1].info {Info::String(ref s)=>s, _ => panic!()};
        return Err(type_error(t.line,t.col,format!(
            "in x.{}:\n  type of x: {}",slot,typ
        )));
    }
}

fn type_check_node(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    match t.value {
        Symbol::Item => {
            match t.info {
                Info::Int(_) => return Ok(Type::Atomic(self.tab.type_int.clone())),
                Info::String(_) => return Ok(Type::Atomic(self.tab.type_string.clone())),
                Info::Id(ref id) => return self.type_check_variable(t,id),
                _ => unimplemented!()
            }
        },
        Symbol::False | Symbol::True => {
            return Ok(Type::Atomic(self.tab.type_bool.clone()));
        },
        Symbol::Plus | Symbol::Minus | Symbol::Ast | Symbol::Div |
        Symbol::Pow
        => {
            return self.type_check_binary_operator(env,t);
        },
        Symbol::Lt | Symbol::Le | Symbol::Gt | Symbol::Ge
        => {
            return self.type_check_comparison(env,t);
        },
        Symbol::Eq | Symbol::Ne => {
            return self.type_check_eq(env,t);
        },
        Symbol::And | Symbol::Or => {
            return self.type_check_logical_operator(env,t);
        },
        Symbol::Cond => {
            return self.type_check_if_expression(env,t);
        },
        Symbol::Index => {
            return self.type_check_operator_index(env,t);
        },
        Symbol::Range => {
            return self.type_check_range(env,t);
        },
        Symbol::Block => {
            return self.type_check_block(env,t);
        },
        Symbol::Let => {
            return self.type_check_let(env,t);
        },
        Symbol::Assignment => {
            self.type_check_assignment(env,t)?;
            return Ok(Type::Atomic(self.tab.type_unit.clone()));
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
        Symbol::If => {
            return self.type_check_if_statement(env,t);
        },
        Symbol::While => {
            return self.type_check_while_statement(env,t);
        },
        Symbol::For => {
            return self.type_check_for_statement(env,t);
        },
        Symbol::Statement => {
            self.type_check_node(env,&t.argv()[0])?;
            return Ok(Type::Atomic(self.tab.type_unit.clone()));
        },
        Symbol::Null => {
            return Ok(Type::Atomic(self.tab.type_unit.clone()));
        }
        Symbol::As => {
            let a = t.argv();
            return Ok(self.type_from_signature(env,&a[1])?);
        },
        Symbol::Dot => {
            return self.type_check_dot(env,t);
        },
        _ => {
            unimplemented!("{}",t.value)
        }
    }
}

pub fn type_check(&mut self, t: &AST)
-> Result<(),SemanticError>
{
    let env = Rc::new(Environment{env: None, map: HashMap::new()});
    let _ = self.type_check_node(&env,t)?;
    return Ok(());
}

}


