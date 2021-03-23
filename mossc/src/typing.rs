
use std::rc::Rc;
use std::collections::HashMap;
use parser::{AST, Symbol, Info};
use self::symbol_table::{SymbolTable, SymbolTableNode};
use error::{Error, error, type_error, undefined_type, undefined_symbol};

#[path = "symbol-table.rs"]
pub mod symbol_table;

#[path = "unify.rs"]
mod unify;

#[path = "traits.rs"]
mod traits;

use self::unify::Substitution;
use self::traits::PredicateTable;

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct TypeId(u32);

const VARIADIC: usize = std::usize::MAX;

type Env = Rc<Environment>;
struct Environment {
    env: Option<Env>,
    map: HashMap<String, TypeVariable>
}
impl Environment {
    fn from_sig(env: &Env, sig: &[TypeVariable]) -> Env {
        let mut m: HashMap<String, TypeVariable> = HashMap::new();
        for tv in sig {
            m.insert((&*tv.id).into(), tv.clone());
        }
        Rc::new(Environment {env: Some(env.clone()), map: m})
    }
    fn get(&self, id: &str) -> Option<&TypeVariable> {
        if let Some(value) = self.map.get(id) {
            Some(value)
        } else if let Some(env) = &self.env {
            env.get(id)
        } else {
            None
        }
    }
    fn _contains(&self, ty0: &Rc<str>) -> bool {
        for ty in self.map.values() {
            if Rc::ptr_eq(ty0, &ty.id) {return true;}
        }
        if let Some(env) = &self.env {
            env._contains(ty0)
        } else {
            false
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
    pub fn _type_tuple(&self) -> Type {
        Type::Atom(self.type_tuple.clone())
    }
    pub fn type_range(&self) -> Type {
        Type::Atom(self.type_range.clone())
    }
    pub fn fn_type(&self, argc_min: usize, argc_max: usize,
        arg: Vec<Type>, ret: Type
    ) -> Type {
        Type::Fn(Rc::new(FnType {
            argc_min, argc_max,
            arg_self: self.type_unit(),
            arg, ret
        }))
    }
    pub fn method_type(&self, argc_min: usize, argc_max: usize,
        arg_self: Type, arg: Vec<Type>, ret: Type
    ) -> Type {
        Type::Fn(Rc::new(FnType {
            argc_min, argc_max, arg_self, arg, ret
        }))
    }
    fn list_of(&self, el: Type) -> Type {
        Type::App(Rc::new(vec![self.type_list(), el]))
    }
}

struct TraitTable {
    map: HashMap<String, Rc<str>>,
    trait_ord: Rc<str>,
    trait_eq: Rc<str>,
    trait_add: Rc<str>,
    trait_sub: Rc<str>,
    trait_mul: Rc<str>,
    trait_div: Rc<str>,
    trait_mod: Rc<str>
}
impl TraitTable {
    fn new() -> Self {
        let trait_ord: Rc<str> = Rc::from("Ord");
        let trait_eq: Rc<str> = Rc::from("Eq");
        let trait_add: Rc<str> = Rc::from("Add");
        let trait_sub: Rc<str> = Rc::from("Sub");
        let trait_mul: Rc<str> = Rc::from("Mul");
        let trait_div: Rc<str> = Rc::from("Div");
        let trait_mod: Rc<str> = Rc::from("Mod");
        let mut map = HashMap::new();
        map.insert("Ord".into(), trait_ord.clone());
        map.insert("Eq".into(), trait_eq.clone());
        map.insert("Add".into(), trait_add.clone());
        map.insert("Sub".into(), trait_sub.clone());
        map.insert("Mul".into(), trait_mul.clone());
        map.insert("Div".into(), trait_div.clone());
        map.insert("Mod".into(), trait_mod.clone());
        Self {map,
            trait_ord, trait_eq,
            trait_add, trait_sub, trait_mul, trait_div,
            trait_mod
        }
    }
}

struct Class {
    map: HashMap<String, Type>
}

struct ClassTable {
    map: HashMap<Rc<str>, Class>
}
impl ClassTable {
    fn new(tab: &TypeTable) -> Self {
        let type_var: Rc<str> = Rc::from("T");
        let type_of_push = tab.method_type(1, 1,
            tab.list_of(Type::Atom(type_var.clone())),
            vec![Type::Atom(type_var.clone())],
            tab.type_unit()
        );
        let type_of_push = Type::poly1(type_var, type_of_push);

        let tv1: Rc<str> = Rc::from("X");
        let tv2: Rc<str> = Rc::from("Y");
        let type_of_map = tab.method_type(1, 1,
            tab.list_of(Type::Atom(tv1.clone())),
            vec![tab.fn_type(1, 1,
                vec![Type::Atom(tv1.clone())],
                Type::Atom(tv2.clone())
            )],
            tab.list_of(Type::Atom(tv2.clone()))
        );
        let type_of_map = Type::poly2(tv1, tv2, type_of_map);

        let tv: Rc<str> = Rc::from("T");
        let type_of_filter = tab.method_type(1, 1,
            tab.list_of(Type::Atom(tv.clone())),
            vec![tab.fn_type(1, 1,
                vec![Type::Atom(tv.clone())],
                tab.type_bool()
            )],
            tab.list_of(Type::Atom(tv.clone()))
        );
        let type_of_filter = Type::poly1(tv, type_of_filter);

        let tv: Rc<str> = Rc::from("T");
        let type_of_all = tab.method_type(1, 1,
            tab.list_of(Type::Atom(tv.clone())),
            vec![tab.fn_type(1, 1,
                vec![Type::Atom(tv.clone())],
                tab.type_bool()
            )],
            tab.type_bool()
        );
        let type_of_all = Type::poly1(tv,type_of_all);
        let type_of_any = type_of_all.clone();

        let tv: Rc<str> = Rc::from("T");
        let type_of_shuffle = tab.method_type(0, 0,
            tab.list_of(Type::Atom(tv.clone())),
            vec![],
            tab.list_of(Type::Atom(tv.clone()))
        );
        let type_of_shuffle = Type::poly1(tv, type_of_shuffle);

        let mut list_map = HashMap::new();
        list_map.insert("push".to_string(), type_of_push);
        list_map.insert("map".to_string(), type_of_map);
        list_map.insert("filter".to_string(), type_of_filter);
        list_map.insert("shuffle".to_string(), type_of_shuffle);
        list_map.insert("all".to_string(), type_of_all);
        list_map.insert("any".to_string(), type_of_any);

        let mut class_map = HashMap::new();
        class_map.insert(tab.type_list.clone(), Class {map: list_map});
        Self {map: class_map}
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
        VariableInfo {var: false, ty: ty, kind: VariableKind::Global}
    }
}

pub struct FnType {
    pub argc_min: usize,
    pub argc_max: usize,
    pub arg_self: Type,
    pub arg: Vec<Type>,
    pub ret: Type
}

#[derive(Clone)]
pub enum Bound {
    None, Trait(Rc<str>), Union(Box<[Rc<str>]>)
}

impl std::fmt::Display for Bound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Bound::None => write!(f, "_"),
            Bound::Trait(s) => write!(f, "{}", s),
            _ => todo!()
        }
    }
}

#[derive(Clone)]
pub struct TypeVariable {
    pub id: Rc<str>,
    pub bound: Bound
}

pub struct PolyType {
    pub variables: Rc<Vec<TypeVariable>>,
    pub scheme: Type
}
impl PolyType {
    fn _contains(&self, id: &Rc<str>) -> bool {
        for tv in &*self.variables {
            if Rc::ptr_eq(&tv.id, id) {return true;}
        }
        false
    }
}

#[derive(Clone)]
pub enum Type {
    None, Atom(Rc<str>), Var(TypeId),
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
        None
    }
    pub fn app(a: Vec<Type>) -> Type {
        Type::App(Rc::new(a))
    }
    fn poly1(type_var: Rc<str>, scheme: Type) -> Type {
        Type::Poly(Rc::new(PolyType {
            variables: Rc::new(vec![TypeVariable {
                id: type_var.clone(),
                bound: Bound::None
            }]),
            scheme
        }))
    }
    fn poly2(type_var1: Rc<str>, type_var2: Rc<str>, scheme: Type)
    -> Type
    {
        Type::Poly(Rc::new(PolyType {
            variables: Rc::new(vec![TypeVariable {
                id: type_var1.clone(),
                bound: Bound::None
            }, TypeVariable {
                id: type_var2.clone(),
                bound: Bound::None
            }]),
            scheme
        }))
    }
    fn is_atomic(&self, typ_id: &Rc<str>) -> bool {
        if let Type::Atom(typ) = self {
            Rc::ptr_eq(typ,typ_id)
        } else {
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
        write!(f, "forall[")?;
        let mut first = true;
        for tv in &*self.variables {
            if first {first = false;} else {write!(f, ", ")?;}
            write!(f,"{}", tv.id)?;
        }
        write!(f, "] ")?;
        write!(f, "{}", self.scheme)
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Type::Atom(s) => write!(f, "{}", s),
            Type::Var(id) => write!(f, "_{}", id.0),
            Type::App(v) => {
                write!(f, "{}[", v[0])?;
                let mut first = true;
                for x in &v[1..] {
                    if first {
                        first = false;
                        write!(f, "{}", x)?;
                    } else {
                        write!(f, ", {}", x)?;
                    }
                }
                write!(f, "]")?;
                Ok(())
            },
            Type::Fn(t) => {
                if t.arg.len() == 1 {
                    write!(f, "{} -> {}", &t.arg[0], &t.ret)
                } else {
                    write!(f, "(")?;
                    let mut first = true;
                    for arg in &t.arg {
                        if first {first = false} else {write!(f,", ")?;}
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ") -> {}", &t.ret)
                }
            },
            Type::Poly(p) => {
                write!(f, "{}", p)
            },
            Type::None => {
                write!(f, "_")
            }
        }
    }
}

fn is_atomic_type(ty: &Type, id: &Rc<str>) -> bool {
    if let Type::Atom(ty) = ty {
        return Rc::ptr_eq(ty,id);
    }
    false
}

fn error_text(app: &AppInfo, typ: &Type) -> String {
    match app {
        AppInfo::Add => format!("x + y not defined for x: {}", typ),
        AppInfo::Sub => format!("x - y not defined for x: {}", typ),
        AppInfo::Mul => format!("x*y not defined for x: {}", typ),
        AppInfo::Mod => format!("x%y not defined for x: {}", typ),
        AppInfo::Eq  => format!("x == y not defined for x: {}", typ),
        AppInfo::Fn(index) => format!(
            "function not defined for argument {}: {}", 
            index, typ)
    }
}

enum AppInfo {
    Add, Sub, Mul, Mod, Eq, Fn(u32)
}

struct Constraint {
    typ: Type,
    bound: Bound,
    node: Rc<AST>,
    app: AppInfo
}

pub struct TypeChecker {
    pub symbol_table: SymbolTable,
    ret_stack: Vec<Type>,
    global_context: bool,
    tab: Rc<TypeTable>,
    trait_tab: TraitTable,
    class_tab: ClassTable,
    subs: Substitution,
    types: Vec<Type>,
    type_id_counter: u32,
    constraints: Vec<Constraint>,
    predicate_tab: PredicateTable,
    unify_log: String
}

impl TypeChecker {

pub fn new(tab: &Rc<TypeTable>) -> Self {
    let symbol_table = SymbolTable::new(&tab);
    let trait_tab = TraitTable::new();
    let class_tab = ClassTable::new(tab);
    let predicate_tab = PredicateTable::new(tab, &trait_tab);
    TypeChecker {
        symbol_table,
        ret_stack: Vec::with_capacity(8),
        global_context: true,
        tab: tab.clone(),
        trait_tab, class_tab,
        subs: Substitution::new(),
        types: vec![Type::None],
        type_id_counter: 0,
        constraints: Vec::new(),
        predicate_tab,
        unify_log: String::new()
    }
}

pub fn string(&self, t: &AST) -> String {
    t.string(&self.types)
}

pub fn subs_as_string(&self) -> String {
    format!("{}", self.subs)
}

fn attach_type(&mut self, t: &AST, typ: &Type) {
    t.typ.index.set(self.types.len());
    self.types.push(typ.clone());
}

fn type_from_signature_or_none(&mut self, env: &Env, t: &AST)
-> Result<Type, Error>
{
    if t.value == Symbol::None {
        Ok(Type::None)
    } else {
        self.type_from_signature(env,t)
    }
}

fn new_uniq_type_id(&mut self) -> TypeId {
    let id = self.type_id_counter;
    self.type_id_counter += 1;
    TypeId(id)
}

fn new_uniq_anonymous_type_var(&mut self, _t: &AST) -> Type {
    // Further information about context can be stored
    // in a Vec<(TypeId, Information)>. Lookup does not need to be
    // fast because it is only needed one time in case of type error.
    Type::Var(self.new_uniq_type_id())
}

fn type_from_signature(&mut self, env: &Env, t: &AST)
-> Result<Type, Error>
{
    if t.value == Symbol::None {
        Ok(self.new_uniq_anonymous_type_var(t))
    } else if let Info::Id(id) = &t.info {
        if let Some(tv) = env.get(id) {
            return Ok(Type::Atom(tv.id.clone()));
        }
        Ok(match &id[..] {
            "Unit" => self.tab.type_unit(),
            "Bool" => self.tab.type_bool(),
            "Int" => self.tab.type_int(),
            "Float" => self.tab.type_float(),
            "String" => self.tab.type_string(),
            "Object" => self.tab.type_object(),
            "_" => self.new_uniq_anonymous_type_var(t),
            id => {
                return Err(undefined_type(t.line, t.col, format!(
                    "Unknown type: {}.", id
                )));
            }
        })
    } else if t.value == Symbol::Application {
        let a = t.argv();
        if let Info::Id(id) = &a[0].info {
            if id == "List" {
                let parameter = self.type_from_signature(env, &a[1])?;
                Ok(self.tab.list_of(parameter))
            } else if id == "Tuple" {
                let mut acc: Vec<Type> = Vec::with_capacity(a.len());
                acc.push(Type::Atom(self.tab.type_tuple.clone()));
                for x in &a[1..] {
                    let parameter = self.type_from_signature(env, x)?;
                    acc.push(parameter);
                }
                Ok(Type::app(acc))
            } else {
                Err(undefined_type(t.line, t.col, format!(
                    "undefined type constructor: {}", id)))
            }
        } else {
            panic!();
        }
    } else if t.value == Symbol::Fn {
        let a = t.argv();
        let arg = if a[0].value == Symbol::List {
            let list = a[0].argv();
            let mut arg: Vec<Type> = Vec::with_capacity(list.len());
            for x in list {
                let ty = self.type_from_signature(env, x)?;
                arg.push(ty);
            }
            arg
        } else {
            vec![self.type_from_signature(env, &a[0])?]
        };
        let n = arg.len();
        let ret = self.type_from_signature(env, &a[1])?;
        Ok(Type::Fn(Rc::new(FnType {
            argc_min: n, argc_max: n,
            arg, ret, arg_self: self.tab.type_unit()
        })))
    } else if t.value == Symbol::Unit {
        Ok(self.tab.type_unit())
    } else {
        unimplemented!("{}", t.value)
    }
}

fn type_check_add(&mut self, t: &Rc<AST>,
    type1: Type, type2: Type
) -> Result<Type, Error>
{
    self.constraints.push(Constraint {
        typ: type1.clone(),
        bound: Bound::Trait(self.trait_tab.trait_add.clone()),
        node: t.clone(),
        app: AppInfo::Add
    });
    self.unify_binary_operator(t, &type1, &type2)?;
    Ok(type1)
}

fn type_check_sub(&mut self, t: &Rc<AST>,
    type1: Type, type2: Type
) -> Result<Type, Error>
{
    self.constraints.push(Constraint {
        typ: type1.clone(),
        bound: Bound::Trait(self.trait_tab.trait_sub.clone()),
        node: t.clone(),
        app: AppInfo::Sub
    });
    self.unify_binary_operator(t, &type1, &type2)?;
    Ok(type1)
}

fn type_check_mul(&mut self, t: &Rc<AST>,
    type1: Type, type2: Type
) -> Result<Type, Error>
{
    self.constraints.push(Constraint {
        typ: type1.clone(),
        bound: Bound::Trait(self.trait_tab.trait_mul.clone()),
        node: t.clone(),
        app: AppInfo::Mul
    });
    self.unify_binary_operator(t, &type1, &type2)?;
    Ok(type1)
}

fn type_check_mod(&mut self, t: &Rc<AST>,
    type1: Type, type2: Type
) -> Result<Type, Error>
{
    self.constraints.push(Constraint {
        typ: type1.clone(),
        bound: Bound::Trait(self.trait_tab.trait_mod.clone()),
        node: t.clone(),
        app: AppInfo::Mod
    });
    self.unify_binary_operator(t, &type1, &type2)?;
    Ok(type1)
}

fn unify_binary_operator(&mut self, t: &AST,
    type1: &Type, type2: &Type
) -> Result<(), Error>
{
    match self.unify(type1, type2) {
        Ok(()) => Ok(()),
        _ => Err(type_error(t.line, t.col,
            format!("in x{}y:{}\nNote:\n    x: {},\n    y: {}",
                t.value, self.unify_log(), type1, type2)))
    }
}

fn type_check_binary_operator(&mut self, env: &Env, t: &Rc<AST>)
-> Result<Type, Error>
{
    let a = t.argv();
    let type1 = self.type_check_node(env,&a[0])?;
    let type2 = self.type_check_node(env,&a[1])?;
    if !type1.is_atomic(&self.tab.type_int) &&
       !type1.is_atomic(&self.tab.type_float) &&
       !type2.is_atomic(&self.tab.type_int) &&
       !type2.is_atomic(&self.tab.type_float)
    {
        return if t.value == Symbol::Plus {
            self.type_check_add(t, type1, type2)
        } else if t.value == Symbol::Minus {
            self.type_check_sub(t, type1, type2)
        } else if t.value == Symbol::Ast {
            self.type_check_mul(t, type1, type2)
        } else if t.value == Symbol::Mod {
            self.type_check_mod(t, type1, type2)
        } else {
            Err(type_error(t.line, t.col, format!(
                "x{}y is not defined for x: {}, y: {}.",
                t.value, type1, type2
            )))
        };
    }
    self.unify_binary_operator(t, &type1, &type2)?;
    Ok(type1)
}

fn type_check_range(&mut self, env: &Env, t: &AST)
-> Result<Type, Error>
{
    let a = t.argv();
    let ta = self.type_check_node(env, &a[0])?;
    let tb = self.type_check_node(env, &a[1])?;
    let td = self.type_check_node(env, &a[2])?;
    let range = self.tab.type_range();
    Ok(Type::app(vec![range, ta, tb, td]))
}

fn index_homogeneous(&mut self, t: &AST, ty_index: &Type, ty: Type)
-> Result<Type, Error>
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
    } else if is_atomic_type(&ty_index, &tab.type_int) {
        return Ok(ty);
    } else if is_atomic_type(&ty_index, &tab.type_object) {
        return Ok(tab.type_object());
    }
    Err(type_error(t.line, t.col, format!(
        "a[i] is not defined for i: {}.", ty_index)))
}

fn type_check_operator_index(&mut self, env: &Env, t: &AST)
-> Result<Type, Error>
{
    let a = t.argv();
    if a.len() > 2 {
        return Err(type_error(t.line, t.col, String::from(
            "in a[...]: expected only one index."
        )));
    }

    let ty_seq = self.type_check_node(env, &a[0])?;
    let mut ty_index = self.type_check_node(env, &a[1])?;
    
    if let Type::Var(_) = ty_index {
        let type_int = self.tab.type_int();
        match self.subs.unify(&ty_index, &type_int, &mut None) {
            Ok(()) => {ty_index = type_int},
            _ => return Err(type_error(t.line, t.col, format!(
                "expected type int,\n  found type {}.", ty_index)))
        }
    }

    if let Some(a) = ty_seq.is_app(&self.tab.type_list) {
        return self.index_homogeneous(t, &ty_index, a[0].clone());
    } else if let Type::Var(_) = &ty_seq {
        let tv = self.new_uniq_anonymous_type_var(t);
        if let Ok(()) = self.subs.unify(&ty_seq,
            &self.tab.list_of(tv.clone()), &mut None
        ) {
            return Ok(tv);
        }
    } else if let Some(_a) = ty_seq.is_app(&self.tab.type_tuple) {
        todo!();
    }

    Err(type_error(t.line, t.col, format!(
        "expected a in a[i] of indexable type,\n  found type {}.",
        ty_seq
    )))
}

fn trait_is_subset(&self, a: &Bound, b: &Bound) -> bool {
    if let Bound::Trait(a) = a {
        if let Bound::Trait(b) = b {
            Rc::ptr_eq(a,b)
        } else {
            false
        }
    } else {
        false
    }
}

fn poly_tv_has_trait(&self, env: &Env, typ: &Type, trait_sig: &Bound) -> bool {
    if let Type::Atom(typ) = typ {
        if let Some(tv) = env.get(typ) {
            self.trait_is_subset(trait_sig, &tv.bound)
        } else {
            false
        }
    } else {
        false
    }
}

fn type_check_comparison(&mut self, env: &Env, t: &AST)
-> Result<Type, Error>
{
    let a = t.argv();
    let type1 = self.type_check_node(env, &a[0])?;
    let type2 = self.type_check_node(env, &a[1])?;
    if !type1.is_atomic(&self.tab.type_int) &&
       !type1.is_atomic(&self.tab.type_float) &&
       !type2.is_atomic(&self.tab.type_int) &&
       !type2.is_atomic(&self.tab.type_float)
    {
        let ord = Bound::Trait(self.trait_tab.trait_ord.clone());
        if !self.poly_tv_has_trait(env, &type1, &ord) &&
           !self.poly_tv_has_trait(env, &type2, &ord)
        {
            return Err(type_error(t.line,t.col,format!(
                "x{}y is not defined for x: {}, y: {}.",
                t.value, type1, type2
            )));
        }
    }
    match self.unify(&type1, &type2) {
        Ok(()) => Ok(self.tab.type_bool()),
        _ => Err(type_error(t.line, t.col, self.unify_log()))
    }
}

fn type_check_eq(&mut self, env: &Env, t: &Rc<AST>)
-> Result<Type, Error>
{
    let a = t.argv();
    let type1 = self.type_check_node(env, &a[0])?;
    let type2 = self.type_check_node(env, &a[1])?;

    self.constraints.push(Constraint {
        typ: type1.clone(),
        bound: Bound::Trait(self.trait_tab.trait_eq.clone()),
        node: t.clone(),
        app: AppInfo::Eq
    });
    match self.unify(&type1, &type2) {
        Ok(()) => Ok(self.tab.type_bool()),
        _ => Err(type_error(t.line, t.col, self.unify_log()))
    }
}

fn type_check_logical_operator(&mut self, env: &Env, t: &AST)
-> Result<Type, Error>
{
    let a = t.argv();
    let type1 = self.type_check_node(env, &a[0])?;
    let type2 = self.type_check_node(env, &a[1])?;
    let type_bool = self.tab.type_bool();
    match self.unify(&type1, &type_bool) {
        Ok(()) => {},
        _ => return Err(type_error(t.line, t.col, self.unify_log()))
    }
    match self.unify(&type2, &type_bool) {
        Ok(()) => {},
        _ => return Err(type_error(t.line, t.col, self.unify_log()))
    }
    Ok(type_bool)
}

fn type_check_not(&mut self, env: &Env, t: &AST)
-> Result<Type, Error>
{
    let a = t.argv();
    let typ = self.type_check_node(env, &a[0])?;
    let typ_bool = self.tab.type_bool();
    match self.unify(&typ, &typ_bool) {
        Ok(()) => {},
        _ => return Err(type_error(t.line, t.col, self.unify_log()))
    }
    Ok(typ_bool)
}

fn type_check_if_expression(&mut self, env: &Env, t: &AST)
-> Result<Type, Error>
{
    let a = t.argv();
    let type0 = self.type_check_node(env, &a[0])?;
    let type1 = self.type_check_node(env, &a[1])?;
    let type2 = self.type_check_node(env, &a[2])?;
    let type_bool = self.tab.type_bool();
    match self.unify(&type0, &type_bool) {
        Ok(()) => {},
        _ => return Err(type_error(t.line, t.col, self.unify_log()))
    }
    match self.unify(&type1, &type2) {
        Ok(()) => {},
        _ => return Err(type_error(t.line, t.col, self.unify_log()))
    }
    Ok(type1)
}

fn type_check_block(&mut self, env: &Env, t: &AST)
-> Result<Type, Error>
{
    let a = t.argv();
    let n = a.len();
    if n == 0 {
        return Ok(self.tab.type_unit());
    }
    for i in 0..n-1 {
        let _ = self.type_check_node(env, &a[i])?;
    }
    let block_type = self.type_check_node(env, &a[n-1])?;
    Ok(block_type)
}

fn type_check_let(&mut self, env_rec: &Env, t: &AST)
-> Result<Type, Error>
{
    let is_var = match t.info {Info::Var => true, _ => false};

    let environment;
    let (env, sig) = if let Info::TypeVars(type_variables) = &t.info {
        let sig = self.poly_sig(type_variables);
        environment = Environment::from_sig(env_rec, &sig);
        (&environment, Some(sig))
    } else {
        (env_rec, None)
    };

    let a = t.argv();
    let id = match &a[0].info {Info::Id(id) => id.clone(), _ => unreachable!()};
    let ty = self.type_from_signature_or_none(env, &a[1])?;
    let ty_expr = self.type_check_node(env, &a[2])?;

    let mut ty_of_id = if let Type::None = ty {
        ty_expr
    } else {
        match self.unify(&ty, &ty_expr) {
            Ok(()) => ty,
            _ => return Err(type_error(t.line, t.col, self.unify_log()))
        }
    };
    let global = self.global_context;

    if let Some(sig) = sig {
        ty_of_id = Type::Poly(Rc::new(PolyType {
            variables: Rc::new(sig), scheme: ty_of_id
        }));
    }
    self.symbol_table.variable_binding(global, is_var, id, ty_of_id);

    Ok(self.tab.type_unit())
}

fn type_check_index_assignment(&mut self, env: &Env, t: &AST)
-> Result<(), Error>
{
    let a = t.argv();
    if a[0].value != Symbol::Index {
        panic!();
    }
    let type1 = self.type_check_node(env, &a[0])?;
    let type2 = self.type_check_node(env, &a[1])?;
    match self.unify(&type1, &type2) {
        Ok(()) => Ok(()),
        _ => Err(type_error(t.line, t.col, self.unify_log()))
    }
}

fn type_check_assignment(&mut self, env: &Env, t: &AST)
-> Result<(), Error>
{
    let a = t.argv();
    let id = match &a[0].info {
        Info::Id(id) => id.clone(),
        _ => return self.type_check_index_assignment(env, t)
    };
    let ty_expr = self.type_check_node(env, &a[1])?;

    let index = self.symbol_table.index;
    let node = &mut self.symbol_table.list[index];
    if let Some(variable_info) = node.get(&id) {
        if !variable_info.var {
            return Err(error(t.line, t.col,
                format!("cannot assign twice to '{}'.", id)));
        }
        let ty = variable_info.ty.clone();
        match self.unify(&ty_expr, &ty) {
            Ok(()) => {},
            _ => return Err(type_error(t.line, t.col,
                format!("in assignment:{}", self.unify_log())))
        }
    } else {
        return Err(undefined_symbol(t.line, t.col, id));
    }
    Ok(())
}

fn type_check_variable(&mut self, t: &AST, id: &String)
-> Result<Type, Error>
{
    // self.symbol_table.print();
    if let Some(t) = self.symbol_table.get(id) {
        // println!("{}, kind: {:?}",id,t.kind);
        Ok(t.ty.clone())
    } else {
        Err(undefined_symbol(t.line, t.col, format!("{}", id)))
    }
}

fn type_check_list(&mut self, env: &Env, t: &AST)
-> Result<Type, Error>
{
    let a = t.argv();
    if a.is_empty() {
        return Ok(Type::app(vec![
            self.tab.type_list(),
            self.new_uniq_anonymous_type_var(t)
        ]));
    }
    let ty0 = self.type_check_node(env, &a[0])?;
    let mut upcast = false;
    for x in &a[1..] {
        let ty = self.type_check_node(env, x)?;
        match self.subs.unify(&ty0, &ty, &mut None) {
            Ok(()) => {},
            _ => {upcast = true;}
        }
    }
    Ok(self.tab.list_of(if upcast {self.tab.type_object()} else {ty0}))
}

fn unify_log(&mut self) -> String {
    std::mem::take(&mut self.unify_log)
}

fn unify(&mut self, t1: &Type, t2: &Type) -> Result<(), ()> {
    self.subs.unify(t1, t2, &mut Some(&mut self.unify_log))
}

fn instantiate_rec(&self, typ: &Type,
    mapping: &HashMap<Rc<str>, TypeId>
) -> Type {
    match typ {
        Type::None => Type::None,
        Type::Atom(typ) => {
            match mapping.get(typ) {
                Some(id) => Type::Var(*id),
                None =>  Type::Atom(typ.clone())
            }
        },
        Type::Var(tv) => Type::Var(tv.clone()),
        Type::App(app) => {
            let a: Vec<Type> = app.iter()
                .map(|x| self.instantiate_rec(x, mapping))
                .collect();
            Type::app(a)
        },
        Type::Fn(typ) => {
            let a: Vec<Type> = typ.arg.iter()
                .map(|x| self.instantiate_rec(x, mapping))
                .collect();
            Type::Fn(Rc::new(FnType {
                argc_min: typ.argc_min,
                argc_max: typ.argc_max,
                arg: a,
                arg_self: self.instantiate_rec(&typ.arg_self, mapping),
                ret: self.instantiate_rec(&typ.ret, mapping)
            }))
        },
        Type::Poly(_) => {
            unreachable!()
        }
    }
}

// Instantiating a type forall[T0, T1, ...] F[T0, T1, ...]
// means replacing T0:=_0, T1:=_1, ... where _0, _1, ...
// are new unique type variables. Only the resulting type
// F[_0, _1, ...] is unified with some given type.

// Example, let id: forall[T] T -> T = |x| x.
// What is the type of id(0)?
// We have 0: Int, thus we can deduce
// => id: forall[T] T -> T is applied to 0: Int
// => id: (T -> T)[T:=_0] is applied to 0: Int
// => id: _0 -> _0 is applied to 0: Int
// => _0 must be unified with Int
// => success, unifier is {_0:=Int}
// => id: Int -> Int
// => id(0): Int.

fn instantiate_poly_type(&mut self, poly: &PolyType, t: &Rc<AST>) -> Type {
    let mapping: HashMap<Rc<str>, TypeId> = poly.variables.iter()
        .enumerate().map(|(index, x)| {
        let id = self.new_uniq_type_id();
        if !matches!(x.bound, Bound::None) {
            self.constraints.push(Constraint {
                typ: Type::Var(id),
                bound: x.bound.clone(),
                node: t.clone(),
                app: AppInfo::Fn(index as u32)
            });
        }
        (x.id.clone(), id)
    }).collect();
    self.instantiate_rec(&poly.scheme, &mapping)
}

fn fn_type_from_app(&mut self, t: &AST, argc: usize) -> (Type, Type) {
    let args: Vec<Type> = (0..argc)
        .map(|_| self.new_uniq_anonymous_type_var(t)).collect();
    let ret = self.new_uniq_anonymous_type_var(t);
    let typ = self.tab.fn_type(argc, argc, args, ret.clone());
    (typ, ret)
}

fn type_check_application(&mut self, env: &Env, t: &Rc<AST>)
-> Result<Type, Error>
{
    let a = t.argv();    
    let argv = &a[1..];
    let argc = argv.len();
    let fn_type = self.type_check_node(env, &a[0])?;
    let self_arg_type = if a[0].value == Symbol::Dot {
        let self_arg = &a[0].argv()[0];
        self.types[self_arg.typ.index.get()].clone()
    } else {
        self.tab.type_unit()
    };

    let sig = match &fn_type {
        Type::Poly(poly) => self.instantiate_poly_type(poly, t),
        typ => typ.clone()
    };

    if let Type::Fn(ref sig) = sig {
        if argc < sig.argc_min || argc > sig.argc_max {
            let id = match a[0].info {Info::Id(ref s) => s, _ => panic!()};
            return Err(type_error(t.line, t.col,
                format!("\n  function {} has argument count {}..{},\n  \
                    found application of argument count {}.",
                    id, sig.argc_min, sig.argc_max, argc)
            ));
        }
        for i in 0..argc {
            let ty = self.type_check_node(env, &argv[i])?;
            let j = if i < sig.arg.len() {i} else {sig.arg.len() - 1};
            if sig.arg[j].is_atomic(&self.tab.type_object) {
                continue;
            }
            match self.unify(&sig.arg[j], &ty) {
                Ok(()) => {},
                _ => return Err(type_error(t.line, t.col,
                    format!("Function argument {}: {}.", i, self.unify_log())))
            }
        }
        match self.unify(&sig.arg_self, &self_arg_type) {
            Ok(()) => {},
            _ => return Err(type_error(t.line, t.col,
                format!("Function self argument: {}.", self.unify_log())))
        }
        return Ok(sig.ret.clone());
    } else if sig.is_atomic(&self.tab.type_object) {
        for i in 0..argc {
            let _ty = self.type_check_node(env, &argv[i])?;
        }
        return Ok(self.tab.type_object());
    } else if let Type::Var(tv) = sig {
        let (typ, ret) = self.fn_type_from_app(t, argc);
        return match self.subs.unify_var(tv, &typ, &mut Some(&mut self.unify_log)) {
            Ok(()) => Ok(ret),
            _ => Err(type_error(t.line, t.col, self.unify_log()))
        };
    }
    Err(type_error(t.line, t.col,
        format!("cannot apply a value of type {}.", fn_type)))
}

fn trait_sig(&self, t: &Rc<AST>) -> Bound {
    if let Info::Id(id) = &t.info {
        match self.trait_tab.map.get(id) {
            Some(value) => Bound::Trait(value.clone()),
            None => panic!("unknown trait '{}'", id)
        }
    } else if let Symbol::List = t.value {
        let argv = t.argv();
        let mut acc = Vec::with_capacity(argv.len());
        for item in argv {
            if let Info::Id(id) = &item.info {
                match self.trait_tab.map.get(id) {
                    Some(value) => {acc.push(value.clone());},
                    None => panic!("unknown trait '{}'", id)
                }
            }
        }
        Bound::Union(acc.into_boxed_slice())
    } else {
        unreachable!()
    }
}

fn poly_sig(&mut self, type_variables: &Rc<AST>) -> Vec<TypeVariable> {
    let mut acc: Vec<TypeVariable> = Vec::new();
    let a = type_variables.argv();
    for x in a {
        let (id_node, bound) = match x.value {
            Symbol::Item => (x, Bound::None),
            Symbol::List => {
                let pair = x.argv();
                let sig = self.trait_sig(&pair[1]);
                (&pair[0], sig)
            },
            _ => unreachable!()
        };
        let id: Rc<str> = match &id_node.info {
            Info::Id(id) => Rc::from(&**id),
            _ => unreachable!()
        };
        if !matches!(bound, Bound::None) {
            self.predicate_tab.extend_bound(&bound, Type::Atom(id.clone()));
        }
        acc.push(TypeVariable {id, bound});
    }
    acc
}

fn type_check_function(&mut self, env: &Env, t: &AST)
-> Result<Type, Error>
{
    let header = match t.info {
        Info::FnHeader(ref h) => h,
        _ => unreachable!()
    };

    let mut variables: Vec<(String, VariableInfo)> = Vec::new();

    let ret = self.type_from_signature(env, &header.ret_type)?;
    let mut arg: Vec<Type> = Vec::with_capacity(header.argv.len());
    let argv = &header.argv;
    for i in 0..argv.len() {
        let ty = self.type_from_signature(env, &argv[i].ty)?;
        arg.push(ty.clone());
        variables.push((argv[i].id.clone(), VariableInfo{
            var: false, ty,
            kind: VariableKind::Argument(i + 1)
        }));
    }

    let argv_len = header.argv.len();
    let mut ftype = self.tab.fn_type(argv_len, argv_len,
        arg, ret.clone()
    );
    if let Type::None = ret {
        // pass
    } else {
        if let Some(ref id) = header.id {
            variables.push((id.clone(), VariableInfo {
                var: false, ty: ftype.clone(),
                kind: VariableKind::FnSelf
            }));
        }
    };

    let context = self.symbol_table.index;
    self.symbol_table.index = self.symbol_table.list.len();
    self.symbol_table.list.push(
        SymbolTableNode::from_variables_and_context(
            variables, Some(context)));

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
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
    } else {
        if !ret_type.is_atomic(&self.tab.type_unit) {
            match self.unify(&ret, &ret_type) {
                Ok(()) => {},
                _ => return Err(type_error(t.line, t.col, self.unify_log()))
            }
        }
    }

    header.symbol_table_index.set(self.symbol_table.index);
    self.symbol_table.index = context;
    Ok(ftype)
}

fn type_check_return(&mut self, env: &Env, t: &AST)
-> Result<Type, Error>
{
    let x = &t.argv()[0];
    let ty_ret = self.type_check_node(env, x)?;
    if let Some(ty) = self.ret_stack.last() {
        let ty = ty.clone();
        match self.unify(&ty_ret, &ty) {
            Ok(()) => Ok(self.tab.type_unit()),
            _ => Err(type_error(t.line, t.col, self.unify_log()))
        }
    } else {
        panic!();
    }
}

fn type_check_if_statement(&mut self, env: &Env, t: &AST)
-> Result<Type, Error>
{
    let a = t.argv();
    let n = a.len();
    let mut i = 0;
    while i+1 < n {
        let type_cond = self.type_check_node(env, &a[i])?;
        let type_bool = self.tab.type_bool();
        match self.unify(&type_cond, &type_bool) {
            Ok(()) => {},
            _ => return Err(type_error(t.line, t.col, self.unify_log()))
        }
        self.type_check_node(env, &a[i+1])?;
        i += 2;
    }
    if n%2 != 0 {
        self.type_check_node(env, &a[n-1])?;
    }
    Ok(self.tab.type_unit())
}

fn type_check_while_statement(&mut self, env: &Env, t: &AST)
-> Result<Type, Error>
{
    let a = t.argv();
    let type_cond = self.type_check_node(env, &a[0])?;
    let type_bool = self.tab.type_bool();
    match self.unify(&type_cond, &type_bool) {
        Ok(()) => {},
        _ => return Err(type_error(t.line, t.col, self.unify_log()))
    }
    self.type_check_node(env, &a[1])?;
    Ok(self.tab.type_unit())
}

fn iter_element(&mut self, iterable: &Type, t: &AST)
-> Result<Type, Error>
{
    if let Some(a) = iterable.is_app(&self.tab.type_list) {
        return Ok(a[0].clone());
    } else if let Some(a) = iterable.is_app(&self.tab.type_range) {
        if a[0].is_atomic(&self.tab.type_int) &&
           a[2].is_atomic(&self.tab.type_unit)
        {
            return match self.unify(&a[0], &a[1]) {
                Ok(()) => Ok(a[0].clone()),
                _ => Err(type_error(t.line, t.col, self.unify_log()))
            };
        }
    }
    Err(type_error(t.line,t.col,format!(
        "{} is not iterable", iterable)))
}

fn type_check_for_statement(&mut self, env: &Env, t: &AST)
-> Result<Type, Error>
{
    let a = t.argv();
    let ty_range = self.type_check_node(env, &a[1])?;
    let typ = self.iter_element(&ty_range, &a[1])?;
    let id = match &a[0].info {Info::Id(id) => id.clone(), _ => panic!()};
    let global = self.global_context;
    self.symbol_table.variable_binding(global, false, id, typ);
    self.type_check_node(env, &a[2])?;
    Ok(self.tab.type_unit())    
}

fn class(&self, typ: &Type) -> Option<&Class> {
    let id = match typ {
        Type::Atom(id) => id,
        Type::App(app) => match &app[0] {
            Type::Atom(id) => id,
            _ => return None
        },
        _ => return None
    };
    self.class_tab.map.get(id)
}

fn type_check_dot(&mut self, env: &Env, t: &AST)
-> Result<Type, Error>
{
    let a = t.argv();
    let typ = self.type_check_node(env, &a[0])?;
    let slot = match a[1].info {Info::String(ref s) => s, _ => panic!()};
    if typ.is_atomic(&self.tab.type_object) {
        Ok(self.tab.type_object())
    } else {
        if let Some(class) = self.class(&typ) {
            if let Some(slot_type) = class.map.get(slot) {
                return Ok(slot_type.clone());
            }
        }
        Err(type_error(t.line, t.col, format!(
            "in x.{}:\n  type of x: {}", slot, typ)))
    }
}

fn type_check_node(&mut self, env: &Env, t: &Rc<AST>)
-> Result<Type, Error>
{
    let typ = self.type_check_node_plain(env, t)?;
    self.attach_type(t, &typ);
    Ok(typ)
}

fn type_check_node_plain(&mut self, env: &Env, t: &Rc<AST>)
-> Result<Type, Error>
{
    match t.value {
        Symbol::Item => {
            match t.info {
                Info::Int(_) => Ok(self.tab.type_int()),
                Info::String(_) => Ok(self.tab.type_string()),
                Info::Id(ref id) => self.type_check_variable(t, id),
                _ => unimplemented!()
            }
        },
        Symbol::False | Symbol::True => {
            Ok(self.tab.type_bool())
        },
        Symbol::Plus | Symbol::Minus | Symbol::Ast | Symbol::Div |
        Symbol::Pow | Symbol::Idiv | Symbol::Mod
        => {
            self.type_check_binary_operator(env, t)
        },
        Symbol::Lt | Symbol::Le | Symbol::Gt | Symbol::Ge
        => {
            self.type_check_comparison(env, t)
        },
        Symbol::Eq | Symbol::Ne => {
            self.type_check_eq(env, t)
        },
        Symbol::And | Symbol::Or => {
            self.type_check_logical_operator(env, t)
        },
        Symbol::Not => {
            self.type_check_not(env, t)
        },
        Symbol::Cond => {
            self.type_check_if_expression(env, t)
        },
        Symbol::Index => {
            self.type_check_operator_index(env, t)
        },
        Symbol::Range => {
            self.type_check_range(env, t)
        },
        Symbol::Block => {
            self.type_check_block(env, t)
        },
        Symbol::Let => {
            self.type_check_let(env, t)
        },
        Symbol::Assignment => {
            self.type_check_assignment(env, t)?;
            Ok(self.tab.type_unit())
        },
        Symbol::List => {
            self.type_check_list(env, t)
        },
        Symbol::Application => {
            self.type_check_application(env, t)
        },
        Symbol::Function => {
            self.type_check_function(env, t)
        },
        Symbol::Return => {
            self.type_check_return(env, t)
        },
        Symbol::If => {
            self.type_check_if_statement(env, t)
        },
        Symbol::While => {
            self.type_check_while_statement(env, t)
        },
        Symbol::For => {
            self.type_check_for_statement(env, t)
        },
        Symbol::Statement => {
            self.type_check_node(env, &t.argv()[0])?;
            Ok(self.tab.type_unit())
        },
        Symbol::Null => {
            Ok(self.tab.type_unit())
        }
        Symbol::As => {
            let a = t.argv();
            self.type_check_node(env, &a[0])?;
            Ok(self.type_from_signature(env, &a[1])?)
        },
        Symbol::Dot => {
            self.type_check_dot(env, t)
        },
        Symbol::Break => {
            Ok(self.tab.type_unit())
        },
        _ => {
            unimplemented!("{}", t.value)
        }
    }
}

pub fn apply_types(&mut self) {
    for typ in &mut self.types {
        *typ = self.subs.apply(typ);
    }
}

pub fn check_constraints(&self) -> Result<(), Error> {
    for constraint in &self.constraints {
        let typ = self.subs.apply(&constraint.typ);
        if let Type::Var(_) = typ {
            // I would like to have this because of convenience.
            // Can this result in any unsoundness?
            // Say, there are conflicting traits, or equivalently,
            // the trait is the empty set?
            return Ok(());
        }

        let bound = &constraint.bound;
        if !self.predicate_tab.apply_bound(&bound, &typ) {
            let node = &constraint.node;
            let err = error_text(&constraint.app, &typ);
            return Err(type_error(node.line, node.col, format!(
                "{}.\nNote:\n    {} not in {}.",
                err, typ, bound
            )));
        }
    }
    Ok(())
}

pub fn type_check(&mut self, t: &Rc<AST>) -> Result<(), Error> {
    let env = Rc::new(Environment {env: None, map: HashMap::new()});
    let _ = self.type_check_node(&env, t)?;
    Ok(())
}

}

