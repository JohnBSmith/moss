
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
    type_float: Rc<str>,
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
            type_bool: Rc::from("Bool"),
            type_int: Rc::from("Int"),
            type_float: Rc::from("Float"),
            type_string: Rc::from("String"),
            type_object: Rc::from("Object"),
            type_list: Rc::from("List"),
            type_tuple: Rc::from("Tuple"),
            type_range: Rc::from("Range")
        })
    }
    pub fn type_unit(&self) -> Type {
        Type::Atom(self.type_unit.clone())
    }
    pub fn type_bool(&self) -> Type {
        Type::Atom(self.type_bool.clone())
    }
    pub fn type_int(&self) -> Type {
        Type::Atom(self.type_int.clone())
    }
    pub fn type_float(&self) -> Type {
        Type::Atom(self.type_float.clone())
    }
    pub fn type_string(&self) -> Type {
        Type::Atom(self.type_string.clone())
    }
    pub fn type_object(&self) -> Type {
        Type::Atom(self.type_object.clone())
    }
    pub fn type_list(&self) -> Type {
        Type::Atom(self.type_list.clone())
    }
    pub fn type_tuple(&self) -> Type {
        Type::Atom(self.type_tuple.clone())
    }
    pub fn type_range(&self) -> Type {
        Type::Atom(self.type_range.clone())
    }
    pub fn fn_type(&self, argc_min: usize, argc_max: usize,
        arg: Vec<Type>, ret: Type
    ) -> Type {
        Type::Fn(Rc::new(FnType{
            argc_min, argc_max,
            arg_self: self.type_unit(),
            arg, ret
        }))
    }
    fn list_of(&self, el: Type) -> Type {
        Type::App(Rc::new(vec![self.type_list(),el]))
    }
}

#[derive(Debug)]
pub enum VariableKind {
    Global, Local(usize), Context(usize),
    Argument(usize), FnSelf
}

pub struct VariableInfo {
    pub var: bool,
    pub kind: VariableKind,
    pub ty: Type
}

impl VariableInfo {
    fn global(ty: Type) -> Self {
        VariableInfo{var: false, ty: ty, kind: VariableKind::Global}
    }
}

pub struct SymbolTableNode {
    pub context: Option<usize>,
    pub variables: Vec<(String,VariableInfo)>,
    pub local_count: usize,
    pub context_count: usize
}

impl SymbolTableNode {
    pub fn count_context(&self) -> usize {
        self.context_count
    }
    pub fn get(&self, id: &str) -> Option<&VariableInfo> {
        for (s,info) in &self.variables {
            if s==id {return Some(info);}
        }
        return None;
    }
    pub fn get_index(&self, id: &str) -> Option<usize> {
        for (index,(s,_)) in self.variables.iter().enumerate() {
            if s==id {return Some(index);}
        }
        return None;
    }
    pub fn contains(&self, id: &str) -> bool {
        return self.get(id).is_some();
    }
    pub fn push_context(&mut self, id: &str, typ: Type) {
        self.variables.push((id.into(),VariableInfo{
            var: true, ty: typ,
            kind: VariableKind::Context(self.context_count),
        }));
        self.context_count +=1;
    }
}

pub struct SymbolTable {
    pub list: Vec<SymbolTableNode>,
    pub index: usize
}

impl SymbolTable {
    pub fn new(tab: &TypeTable) -> Self {
        let type_of_print = tab.fn_type(0,VARIADIC,
            vec![tab.type_object()],
            tab.type_unit()
        );

        let type_var: Rc<str> = Rc::from("T");
        let type_of_len = tab.fn_type(1,1,
            vec![tab.list_of(Type::Atom(type_var.clone()))],
            tab.type_int()
        );
        let type_of_len = Type::poly1(type_var,type_of_len);

        let type_of_str = tab.fn_type(1,1,
            vec![tab.type_object()],
            tab.type_string()
        );

        let type_of_list = tab.fn_type(1,1,
            vec![Type::app(vec![tab.type_range(),
                tab.type_int(),
                tab.type_int(),
                tab.type_unit()
            ])],
            tab.list_of(tab.type_int())
        );

        let type_of_iter = tab.fn_type(1,1,
            vec![tab.type_object()],
            tab.type_object()
        );
        
        let type_of_input = tab.fn_type(0,1,
            vec![tab.type_string()],
            tab.type_string()
        );
        
        let type_of_int = tab.fn_type(1,1,
            vec![tab.type_string()],
            tab.type_int()
        );

        let variables: Vec<(String,VariableInfo)> = vec![
            ("print".into(),VariableInfo::global(type_of_print)),
            ("len".into(),VariableInfo::global(type_of_len)),
            ("str".into(),VariableInfo::global(type_of_str)),
            ("list".into(),VariableInfo::global(type_of_list)),
            ("iter".into(),VariableInfo::global(type_of_iter)),
            ("input".into(),VariableInfo::global(type_of_input)),
            ("int".into(),VariableInfo::global(type_of_int))
        ];

        let node = SymbolTableNode{
            context: None, variables,
            local_count: 0, context_count: 0
        };
        let table = SymbolTable{
            index: 0, list: vec![node]
        };
        return table;
    }

    fn get_rec(&mut self, index: usize, key: &str) -> Option<(usize,usize)> {
        let node = &mut self.list[index];
        if let Some(i) = node.get_index(key) {
            return Some((index,i));
        }else if let Some(context) = node.context {
            if let Some(t) = self.get_rec(context,key) {
                let info = &self.list[t.0].variables[t.1].1;
                if let VariableKind::Global = info.kind {
                    return Some(t);
                }
                let typ = info.ty.clone();
                let node = &mut self.list[index];
                node.push_context(key,typ);
                return Some((index,node.variables.len()-1));
            }else{
                return None;
            }
        }else{
            return None;
        }
    }
    
    pub fn get(&mut self, key: &str) -> Option<&VariableInfo> {
        let index = self.index;
        return match self.get_rec(index,key) {
            Some(t) => Some(&self.list[t.0].variables[t.1].1),
            None => None
        };
    }
    pub fn node(&self) -> &SymbolTableNode {
        &self.list[self.index]
    }
    pub fn context_node(&self) -> Option<&SymbolTableNode> {
        let node = &self.list[self.index];
        if let Some(context) = node.context {
            return Some(&self.list[context]);
        }else{
            return None;
        }
    }
    pub fn local_count(&self) -> usize {
        let index = self.index;
        return self.list[index].local_count;
    }
    fn print(&self){
        println!("index: {}",self.index);
        for (i,x) in self.list.iter().enumerate() {
            print!("[{}] ",i);
            for (id,_) in &x.variables {
                print!("{}, ",id);
            }
            println!();
        }
        println!();
    }

    pub fn variable_binding(&mut self, global: bool, is_var: bool,
        id: String, typ: Type
    ) {
        let index = self.index;
        let node = &mut self.list[index];
        if node.contains(&id) {
            panic!();
        }
        let kind = if global {
            VariableKind::Global
        }else{
            node.local_count+=1;
            VariableKind::Local(node.local_count-1)
        };
        node.variables.push((id,VariableInfo{
            var: is_var, ty: typ, kind
        }));
    }
}

const VARIADIC: usize = std::usize::MAX;

pub struct FnType {
    pub argc_min: usize,
    pub argc_max: usize,
    pub arg_self: Type,
    pub arg: Vec<Type>,
    pub ret: Type
}

pub struct TypeVariable {
    pub id: Rc<str>,
    pub class: Type
}

pub struct PolyType {
    pub variables: Rc<Vec<TypeVariable>>,
    pub scheme: Type
}
impl PolyType {
    fn contains(&self, id: &Rc<str>) -> bool {
        for tv in &*self.variables {
            if Rc::ptr_eq(&tv.id,id) {return true;}
        }
        return false;
    }
}

#[derive(Clone)]
pub enum Type {
    None, Atom(Rc<str>), Var(Rc<str>),
    App(Rc<Vec<Type>>), Fn(Rc<FnType>),
    Poly(Rc<PolyType>)
}

impl Type {
    fn is_app(&self, id: &Rc<str>) -> Option<&[Type]> {
        if let Type::App(a) = self {
            if let Type::Atom(f) = &a[0] {
                if Rc::ptr_eq(f,id) {return Some(&a[1..]);}
            }
        }
        return None;
    }
    pub fn app(a: Vec<Type>) -> Type {
        Type::App(Rc::new(a))
    }
    fn poly1(type_var: Rc<str>, scheme: Type) -> Type {
        Type::Poly(Rc::new(PolyType{
            variables: Rc::new(vec![TypeVariable{
                id: type_var.clone(),
                class: Type::None
            }]),
            scheme
        }))
    }
    fn is_atomic(&self, typ_id: &Rc<str>) -> bool {
        if let Type::Atom(typ) = self {
            Rc::ptr_eq(typ,typ_id)
        }else{
            false
        }
    }
    fn contains_var(&self) -> bool {
        match self {
            Type::None | Type::Atom(_) => false,
            Type::Var(_) => true,
            Type::App(app) => app.iter().any(|x| x.contains_var()),
            Type::Fn(typ) =>
                typ.ret.contains_var() ||
                typ.arg.iter().any(|x| x.contains_var()),
            Type::Poly(_) => true
        }
    }
}

impl std::fmt::Display for PolyType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"forall[")?;
        let mut first = true;
        for tv in &*self.variables {
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
            Type::Atom(s) => write!(f,"{}",s),
            Type::Var(s) => write!(f,"{}",s),
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
                    write!(f,"(")?;
                    let mut first = true;
                    for arg in &t.arg {
                        if first {first = false} else {write!(f,", ")?;}
                        write!(f,"{}",arg)?;
                    }
                    write!(f,")=>{}",&t.ret)
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
    format!("\n    expected type {}\n    found    type {}",expected,given)
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
        Type::Atom(t1) => {
            match t2 {
                Type::Atom(t2) => TypeCmp::from(t1==t2),
                Type::None => TypeCmp::None,
                Type::App(_) => TypeCmp::False,
                Type::Fn(_) => TypeCmp::False,
                _ => panic!()
            }
        },
        Type::App(ref p1) => {
            match t2 {
                Type::None => TypeCmp::None,
                Type::Atom(_) => TypeCmp::False,
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
    if let Type::Atom(ty) = ty {
        return Rc::ptr_eq(ty,id);
    }
    return false;
}

struct TypeId(Rc<str>);

impl PartialEq for TypeId {
    fn eq(&self, y: &Self) -> bool {
        Rc::ptr_eq(&self.0,&y.0)
    }
}
impl Eq for TypeId {}

impl std::hash::Hash for TypeId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ((&*self.0) as *const str as *const () as usize).hash(state);
    }
}

struct Substitution {
    map: HashMap<TypeId,Type>
}
impl Substitution {
    fn new() -> Self {
        Self{map: HashMap::new()}
    }
    fn apply(&self, typ: &Type) -> Type {
        match typ {
            Type::None => Type::None,
            Type::Atom(typ) => {
                Type::Atom(typ.clone())
            },
            Type::Var(tv) => {
                let id = TypeId(tv.clone());
                let subs = match self.map.get(&id) {
                    Some(value) => value,
                    None => return Type::Var(id.0)
                };
                if let Type::Atom(typ) = subs {
                    return Type::Atom(typ.clone());
                }else if subs.contains_var() {
                    return self.apply(subs);
                }else{
                    return subs.clone();
                }
            },
            Type::App(app) => {
                Type::app(app.iter().map(|x|
                    self.apply(x)).collect::<Vec<Type>>())
            },
            Type::Fn(typ) => {
                Type::Fn(Rc::new(FnType{
                    argc_min: typ.argc_min,
                    argc_max: typ.argc_max,
                    arg_self: self.apply(&typ.arg_self),
                    arg: typ.arg.iter().map(|x| self.apply(x)).collect(),
                    ret: self.apply(&typ.ret)
                }))
            },
            Type::Poly(poly_type) => {
                Type::Poly(Rc::new(PolyType{
                    variables: poly_type.variables.clone(),
                    scheme: self.apply(&poly_type.scheme)
                }))
            }
        }
    }
}

impl std::fmt::Display for Substitution {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut a: Vec<(&TypeId,&Type)> = self.map.iter().collect();
        a.sort_by_key(|t| &(t.0).0);
        for (key,value) in &a {
            writeln!(f,"{} := {},",key.0,value).ok();
        }
        return Ok(());
    }
}

pub struct TypeChecker {
    pub symbol_table: SymbolTable,
    ret_stack: Vec<Type>,
    global_context: bool,
    tab: Rc<TypeTable>,
    subs: Substitution,
    types: Vec<Type>,
    tv_counter: u32
}

impl TypeChecker {

pub fn new(tab: &Rc<TypeTable>) -> Self {
    let symbol_table = SymbolTable::new(&tab);
    return TypeChecker{
        symbol_table,
        ret_stack: Vec::with_capacity(8),
        global_context: true,
        tab: tab.clone(),
        subs: Substitution::new(),
        types: vec![Type::None],
        tv_counter: 0
    };
}

pub fn string(&self, t: &AST) -> String {
    return t.string(&self.types);
}

pub fn subs_as_string(&self) -> String {
    return format!("{}",self.subs);
}

fn attach_type(&mut self, t: &AST, typ: &Type) {
    t.typ.index.set(self.types.len());
    self.types.push(typ.clone());
}

fn type_from_signature_or_none(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    if t.value == Symbol::None {
        return Ok(Type::None);
    }else{
        return self.type_from_signature(env,t);
    }
}

fn new_uniq_anonymous_type_var(&mut self, _t: &AST) -> Type {
    self.tv_counter+=1;
    return Type::Var(Rc::from(format!("v{}",self.tv_counter)));
}

fn type_from_signature(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    if t.value == Symbol::None {
        return Ok(self.new_uniq_anonymous_type_var(t));
    }else if let Info::Id(id) = &t.info {
        if let Some(ty) = env.get(id) {
            return Ok(Type::Atom(ty));
        }
        return Ok(match &id[..] {
            "Unit" => self.tab.type_unit(),
            "Bool" => self.tab.type_bool(),
            "Int" => self.tab.type_int(),
            "Float" => self.tab.type_float(),
            "String" => self.tab.type_string(),
            "Object" => self.tab.type_object(),
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
                return Ok(self.tab.list_of(parameter));
            }else if id=="Tuple" {
                let mut v: Vec<Type> = Vec::with_capacity(a.len());
                v.push(Type::Atom(self.tab.type_tuple.clone()));
                for x in &a[1..] {
                    let parameter = self.type_from_signature(env,x)?;
                    v.push(parameter);
                }
                return Ok(Type::app(v));
            }else{
                panic!();
            }
        }else{
            panic!();
        }
    }else if t.value == Symbol::Fn {
        let a = t.argv();
        let arg = if a[0].value == Symbol::List {
            let list = a[0].argv();
            let mut arg: Vec<Type> = Vec::with_capacity(list.len());
            for x in list {
                let ty = self.type_from_signature(env,x)?;
                arg.push(ty);
            }
            arg
        }else{
            vec![self.type_from_signature(env,&a[0])?]
        };
        let n = arg.len();
        let ret = self.type_from_signature(env,&a[1])?;
        return Ok(Type::Fn(Rc::new(FnType {
            argc_min: n, argc_max: n,
            arg, ret, arg_self: self.tab.type_unit()
        })));
    }else if t.value == Symbol::Unit {
        return Ok(self.tab.type_unit());
    }else{
        unimplemented!("{}",t.value);
    }
}

fn type_check_binary_operator(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let type1 = self.type_check_node(env,&a[0])?;
    let type2 = self.type_check_node(env,&a[1])?;
    if !type1.is_atomic(&self.tab.type_int) &&
       !type1.is_atomic(&self.tab.type_float) &&
       !type2.is_atomic(&self.tab.type_int) &&
       !type2.is_atomic(&self.tab.type_float)
    {
        if t.value == Symbol::Plus && type1.is_app(&self.tab.type_list).is_some() {
            // pass
        }else{
            return Err(type_error(t.line,t.col,format!(
                "x{}y is not defined for x: {}, y: {}.",t.value,type1,type2
            )));
        }
    }
    match self.unify(env,&type1,&type2) {
        Ok(()) => {},
        Err(err) => return Err(type_error(t.line,t.col,
            format!("in x{}y:{}\nNote:\n    x: {},\n    y: {}",
                t.value,err,&type1,&type2)))
    }
    return Ok(type1);
}

fn type_check_range(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let ta = self.type_check_node(env,&a[0])?;
    let tb = self.type_check_node(env,&a[1])?;
    let td = self.type_check_node(env,&a[2])?;
    let range = self.tab.type_range();
    return Ok(Type::app(vec![range,ta,tb,td]));
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
                    return Ok(tab.list_of(ty));
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
    if !type1.is_atomic(&self.tab.type_int) &&
       !type1.is_atomic(&self.tab.type_float) &&
       !type2.is_atomic(&self.tab.type_int) &&
       !type2.is_atomic(&self.tab.type_float)
    {
        return Err(type_error(t.line,t.col,format!(
            "x{}y is not defined for x: {}, y: {}.",t.value,type1,type2
        )));
    }
    return match self.unify(env,&type1,&type2) {
        Ok(()) => Ok(self.tab.type_bool()),
        Err(err) => Err(type_error(t.line,t.col,err))
    };
}

fn type_check_eq(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let type1 = self.type_check_node(env,&a[0])?;
    let type2 = self.type_check_node(env,&a[1])?;
    return match self.unify(env,&type1,&type2) {
        Ok(()) => Ok(self.tab.type_bool()),
        Err(err) => Err(type_error(t.line,t.col,err))
    };
}

fn type_check_logical_operator(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let type1 = self.type_check_node(env,&a[0])?;
    let type2 = self.type_check_node(env,&a[1])?;
    let type_bool = self.tab.type_bool();
    match self.unify(env,&type1,&type_bool) {
        Ok(()) => {},
        Err(err) => return Err(type_error(t.line,t.col,err))
    }
    match self.unify(env,&type2,&type_bool) {
        Ok(()) => {},
        Err(err) => return Err(type_error(t.line,t.col,err))
    }
    return Ok(type_bool);
}

fn type_check_not(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let typ = self.type_check_node(env,&a[0])?;
    let typ_bool = self.tab.type_bool();
    match self.unify(env,&typ,&typ_bool) {
        Ok(()) => {},
        Err(err) => return Err(type_error(t.line,t.col,err))
    }
    return Ok(typ_bool);
}

fn type_check_if_expression(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let type0 = self.type_check_node(env,&a[0])?;
    let type1 = self.type_check_node(env,&a[1])?;
    let type2 = self.type_check_node(env,&a[2])?;
    let type_bool = self.tab.type_bool();
    match self.unify(env,&type0,&type_bool) {
        Ok(()) => {},
        Err(err) => return Err(type_error(t.line,t.col,err))
    }
    match self.unify(env,&type1,&type2) {
        Ok(()) => {},
        Err(err) => return Err(type_error(t.line,t.col,err))
    }
    return Ok(type1);
}

fn type_check_block(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let n = a.len();
    if n==0 {
        return Ok(self.tab.type_unit());
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
    let is_var = match t.info {Info::Var => true, _ => false};
    let a = t.argv();
    let id = match &a[0].info {Info::Id(id) => id.clone(), _ => unreachable!()};
    let ty = self.type_from_signature_or_none(env,&a[1])?;
    let ty_expr = self.type_check_node(env,&a[2])?;

    let ty_of_id = if let Type::None = ty {
        ty_expr
    }else{
        match self.unify(env,&ty,&ty_expr) {
            Ok(()) => ty,
            Err(err) => return Err(type_error(t.line,t.col,err))
        }
    };
    let global = self.global_context;
    self.symbol_table.variable_binding(global,is_var,id,ty_of_id);

    return Ok(self.tab.type_unit());
}

fn type_check_index_assignment(&mut self, env: &Env, t: &AST)
-> Result<(),SemanticError>
{
    let a = t.argv();
    if a[0].value != Symbol::Index {
        panic!();
    }
    let type1 = self.type_check_node(env,&a[0])?;
    let type2 = self.type_check_node(env,&a[1])?;
    return match self.unify(env,&type1,&type2) {
        Ok(()) => Ok(()),
        Err(err) => Err(type_error(t.line,t.col,err))
    };
}

fn type_check_assignment(&mut self, env: &Env, t: &AST)
-> Result<(),SemanticError>
{
    let a = t.argv();
    let id = match &a[0].info {
        Info::Id(id) => id.clone(),
        _ => return self.type_check_index_assignment(env,t)
    };
    let ty_expr = self.type_check_node(env,&a[1])?;

    let index = self.symbol_table.index;
    let node = &mut self.symbol_table.list[index];
    if let Some(variable_info) = node.get(&id) {
        if !variable_info.var {
            return Err(error(t.line,t.col,
                format!("cannot assign twice to '{}'.",id)
            ));
        }
        let ty = variable_info.ty.clone();
        match self.unify(env,&ty_expr,&ty) {
            Ok(()) => {},
            Err(err) => return Err(type_error(t.line,t.col,
                format!("in assignment:{}",err)
            ))
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

fn type_check_list(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    if a.is_empty() {
        return Ok(Type::app(vec![
            self.tab.type_list(),
            self.new_uniq_anonymous_type_var(t)
        ]));
    }
    let ty0 = self.type_check_node(env,&a[0])?;
    for (k,x) in (&a[1..]).iter().enumerate() {
        let ty = self.type_check_node(env,x)?;
        match self.unify(env,&ty0,&ty) {
            Ok(()) => {},
            Err(err) => return Err(type_error(x.line,x.col,
                format!("in list literal at index {}:{}",k+1,err)
            ))
        }
    }
    return Ok(self.tab.list_of(ty0));
}

fn unify_fn(&mut self, env: &Env, f1: &FnType, f2: &FnType)
-> Result<(),String>
{
    self.unify(env,&f1.ret,&f2.ret)?;
    if f1.arg.len() != f2.arg.len() ||
       f1.argc_min != f2.argc_min ||
       f1.argc_max != f2.argc_max
    {
        return Err(format!("mismatch in number of arguments:\n  expected {}\n  found {}",
            f1.arg.len(), f2.arg.len()
        ));
    }
    self.unify(env,&f1.arg_self,&f2.arg_self)?;
    for i in 0..f1.arg.len() {
        self.unify(env,&f1.arg[i],&f2.arg[i])?;
    }
    return Ok(());
}

fn unify_var(&mut self, env: &Env, tv: &Rc<str>, t2: &Type)
-> Result<(),String>
{
    // println!("{} = {}",tv,t2);
    if let Some(t1) = self.subs.map.get(&TypeId(tv.clone())) {
        let t1 = t1.clone();
        return self.unify(env,&t1,t2);
    }else if let Type::Var(tv2) = t2 {
        if Rc::ptr_eq(tv,tv2) {return Ok(());}
        if let Some(t2) = self.subs.map.get(&TypeId(tv2.clone())) {
            let t2 = t2.clone();
            return self.unify_var(env,tv,&t2);
        }
    }
    self.subs.map.insert(TypeId(tv.clone()),t2.clone());
    return Ok(());
}

fn unify(&mut self, env: &Env, t1: &Type, t2: &Type)
-> Result<(),String>
{
    if let Type::Var(tv1) = t1 {
        return self.unify_var(env,tv1,t2);
    }
    if let Type::Var(tv2) = t2 {
        return self.unify_var(env,tv2,t1);
    }
    match t1 {
        Type::Atom(t1) => {
            if let Type::Atom(t2) = t2 {
                if Rc::ptr_eq(t1,t2) {return Ok(());}
            }
            if let Some(t1) = self.subs.map.get(&TypeId(t1.clone())) {
                let t1 = t1.clone();
                return self.unify(env,&t1,t2);
            }
        },
        Type::App(app_t1) => {
            if let Type::App(app_t2) = t2 {
                if app_t1.len() != app_t2.len() {
                    return Err(format!("Mismatch in number of type arguments: \n  expected {}, \n  found {}",
                        t1,t2));
                }
                for i in 0..app_t1.len() {
                    self.unify(env,&app_t1[i],&app_t2[i])?;
                }
                return Ok(());
            }else{
                return Err(type_mismatch(t1,t2));
            }
        },
        Type::Fn(fn_t1) => {
            if let Type::Fn(fn_t2) = t2 {
                return self.unify_fn(env,fn_t1,fn_t2);
            }else{
                return Err(type_mismatch(t1,t2));
            }
        },
        t1 => {
            return Err(format!("Cannot unify {}",t1));
        }
    }
    return Err(type_mismatch(t1,t2));
}

fn instantiate_rec(&self, typ: &Type,
    mapping: &HashMap<Rc<str>,Rc<str>>
) -> Type {
    match typ {
        Type::None => Type::None,
        Type::Atom(typ) => {
            match mapping.get(typ) {
                Some(id) => Type::Var(id.clone()),
                None =>  Type::Atom(typ.clone())
            }           
        },
        Type::Var(tv) => Type::Var(tv.clone()),
        Type::App(app) => {
            let a: Vec<Type> = app.iter()
                .map(|x| self.instantiate_rec(x,mapping))
                .collect();
            Type::app(a)
        },
        Type::Fn(typ) => {
            let a: Vec<Type> = typ.arg.iter()
                .map(|x| self.instantiate_rec(x,mapping))
                .collect();
            Type::Fn(Rc::new(FnType{
                argc_min: typ.argc_min,
                argc_max: typ.argc_max,
                arg: a,
                arg_self: self.instantiate_rec(&typ.arg_self,mapping),
                ret: self.instantiate_rec(&typ.ret,mapping)
            }))
        },
        Type::Poly(_) => {
            unreachable!()
        }
    }
}

fn instantiate_poly_type(&self, poly: &PolyType) -> Type {
    let mapping: HashMap<Rc<str>,Rc<str>> = poly.variables.iter()
        .map(|x| (x.id.clone(),Rc::from(&*x.id))).collect();
    self.instantiate_rec(&poly.scheme,&mapping)
}

fn fn_type_from_app(&mut self, t: &AST, argc: usize) -> (Type,Type) {
    let args: Vec<Type> = (0..argc)
        .map(|_| self.new_uniq_anonymous_type_var(t)).collect();
    let ret = self.new_uniq_anonymous_type_var(t);
    let typ = self.tab.fn_type(argc,argc,args,ret.clone());
    return (typ,ret);
}

fn type_check_application(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let argv = &a[1..];
    let argc = argv.len();
    let fn_type = self.type_check_node(env,&a[0])?;

    let sig = match &fn_type {
        Type::Poly(poly) => self.instantiate_poly_type(poly),
        typ => typ.clone()
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
            let ty = self.type_check_node(env,&argv[i])?;
            let j = if i<sig.arg.len() {i} else {sig.arg.len()-1};
            if sig.arg[j].is_atomic(&self.tab.type_object) {
                continue;
            }
            match self.unify(env,&sig.arg[j],&ty) {
                Ok(()) => {},
                Err(text) => {
                    return Err(type_error(t.line,t.col,
                        format!("Function argument {}: {}.",i,text)
                    ));
                }
            }
        }
        return Ok(sig.ret.clone());
    }else if sig.is_atomic(&self.tab.type_object) {
        for i in 0..argc {
            let _ty = self.type_check_node(env,&argv[i])?;        
        }
        return Ok(self.tab.type_object());
    }else if let Type::Var(tv) = sig {
        let (typ,ret) = self.fn_type_from_app(t,argc);
        return match self.unify_var(env,&tv,&typ) {
            Ok(()) => Ok(ret),
            Err(err) => Err(type_error(t.line,t.col,err))
        };
    }
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

    let mut variables: Vec<(String,VariableInfo)> = Vec::new();

    let (env,sig) = if let Some(type_variables) = &header.type_variables {
        let sig = self.poly_sig(type_variables);
        let env = Environment::from_sig(env_rec,&sig);
        (env,Some(sig))
    }else{
        (env_rec.clone(), None)
    };

    let ret = self.type_from_signature_or_none(&env,&header.ret_type)?;
    let mut arg: Vec<Type> = Vec::with_capacity(header.argv.len());
    let argv = &header.argv;
    for i in 0..argv.len() {
        let ty = self.type_from_signature(&env,&argv[i].ty)?;
        arg.push(ty.clone());
        variables.push((argv[i].id.clone(),VariableInfo{
            var: false, ty,
            kind: VariableKind::Argument(i+1)
        }));
    }

    let argv_len = header.argv.len();
    let mut ftype = self.tab.fn_type(argv_len,argv_len,
        arg, ret.clone()
    );
    if let Type::None = ret {
        // pass
    }else{
        if let Some(ref id) = header.id {
            variables.push((id.clone(),VariableInfo{
                var: false, ty: ftype.clone(),
                kind: VariableKind::FnSelf
            }));
        }
    };

    let context = self.symbol_table.index;
    self.symbol_table.index = self.symbol_table.list.len();
    self.symbol_table.list.push(SymbolTableNode{
        variables,
        context: Some(context),
        local_count: 0, context_count: 0
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
            match self.unify(&env,&ret,&ret_type) {
                Ok(()) => {},
                Err(err) => return Err(type_error(t.line,t.col,err))
            }
        }
    }

    header.symbol_table_index.set(self.symbol_table.index);
    self.symbol_table.index = context;

    if let Some(sig) = sig {
        return Ok(Type::Poly(Rc::new(PolyType{
            variables: Rc::new(sig), scheme: ftype
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
        let ty = ty.clone();
        return match self.unify(env,&ty_ret,&ty) {
            Ok(()) => Ok(self.tab.type_unit()),
            Err(err) => Err(type_error(t.line,t.col,err))
        };
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
        let type_cond = self.type_check_node(env,&a[i])?;
        let type_bool = self.tab.type_bool();
        match self.unify(env,&type_cond,&type_bool) {
            Ok(()) => {},
            Err(err) => {
                return Err(type_error(t.line,t.col,err));
            }
        }
        self.type_check_node(env,&a[i+1])?;
        i+=2;
    }
    if n%2 != 0 {
        self.type_check_node(env,&a[n-1])?;
    }
    return Ok(self.tab.type_unit());
}

fn type_check_while_statement(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let type_cond = self.type_check_node(env,&a[0])?;
    let type_bool = self.tab.type_bool();
    match self.unify(env,&type_cond,&type_bool) {
        Ok(()) => {},
        Err(err) => {
            return Err(type_error(t.line,t.col,err));
        }
    }
    self.type_check_node(env,&a[1])?;
    return Ok(self.tab.type_unit());
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
    return Ok(self.tab.type_unit());
}

fn type_check_dot(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    let a = t.argv();
    let typ = self.type_check_node(env,&a[0])?;
    if typ.is_atomic(&self.tab.type_object) {
        return Ok(self.tab.type_object());
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
    let typ = self.type_check_node_plain(env,t)?;
    self.attach_type(t,&typ);
    return Ok(typ);
}

fn type_check_node_plain(&mut self, env: &Env, t: &AST)
-> Result<Type,SemanticError>
{
    match t.value {
        Symbol::Item => {
            return match t.info {
                Info::Int(_) => Ok(self.tab.type_int()),
                Info::String(_) => Ok(self.tab.type_string()),
                Info::Id(ref id) => self.type_check_variable(t,id),
                _ => unimplemented!()
            };
        },
        Symbol::False | Symbol::True => {
            return Ok(self.tab.type_bool());
        },
        Symbol::Plus | Symbol::Minus | Symbol::Ast | Symbol::Div |
        Symbol::Pow | Symbol::Idiv
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
        Symbol::Not => {
            return self.type_check_not(env,t);
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
            return Ok(self.tab.type_unit());
        },
        Symbol::List => {
            return self.type_check_list(env,t);
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
            return Ok(self.tab.type_unit());
        },
        Symbol::Null => {
            return Ok(self.tab.type_unit());
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

pub fn apply_types(&mut self) {
    for typ in &mut self.types {
        *typ = self.subs.apply(typ);
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

