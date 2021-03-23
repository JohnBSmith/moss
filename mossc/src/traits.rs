
/*
What follows is basically logic programming, i.e. kind of a Prolog
engine. A trait is considered to be a predicate. One abbreviates
"type T has trait P" as P(T).

The rules of a trait P consist of:
leaf: a list of facts,
clauses: a list of clauses of the form
   P(type pattern) :- condition.

Example. Is "[1, 2] + [3, 4]" a correct program? We have
+: forall[T: Add] (T, T) -> T. Thus T = List[Int].

There is a rule
Add(List[X]) :- Condition::None.

Matching Add(List[Int]) against the list of clauses of Add succeeds
at Add(List[X]), we get the unifier {X:=Int}. The are are no
conditions to fulfill, so we are done.
*/

use std::rc::Rc;
use std::collections::HashMap;

use crate::typing::{Type, TypeId, TypeTable, TraitTable, Bound};
use crate::typing::unify::Substitution;

enum Condition {
    None, Bound(Type, Bound)
}

struct Rules {
    leaf: Vec<Type>, 
    clauses: Vec<(Type, Condition)> 
}

type TraitId = Rc<str>;

pub struct PredicateTable {
    map: HashMap<TraitId, Rules>
}

const VAR0: u32 = 10000000;

fn init_table(type_tab: &TypeTable, trait_tab: &TraitTable)
-> PredicateTable
{
    let mut map = HashMap::new();
    map.insert(trait_tab.trait_add.clone(), Rules {leaf: vec![
        Type::Atom(type_tab.type_int.clone()),
        Type::Atom(type_tab.type_float.clone()),
        Type::Atom(type_tab.type_string.clone()),
        Type::Atom(type_tab.type_object.clone())
    ], clauses: vec![
        (type_tab.list_of(Type::Var(TypeId(VAR0))), Condition::None)
    ]});
    map.insert(trait_tab.trait_sub.clone(), Rules {leaf: vec![
        Type::Atom(type_tab.type_int.clone()),
        Type::Atom(type_tab.type_float.clone()),
        Type::Atom(type_tab.type_object.clone())
    ], clauses: vec![]});
    map.insert(trait_tab.trait_mul.clone(), Rules {leaf: vec![
        Type::Atom(type_tab.type_int.clone()),
        Type::Atom(type_tab.type_float.clone()),
        Type::Atom(type_tab.type_object.clone())
    ], clauses: vec![]});
    map.insert(trait_tab.trait_div.clone(), Rules {leaf: vec![
        Type::Atom(type_tab.type_float.clone()),
        Type::Atom(type_tab.type_object.clone())
    ], clauses: vec![]});
    map.insert(trait_tab.trait_eq.clone(), Rules {leaf: vec![
        Type::Atom(type_tab.type_bool.clone()),
        Type::Atom(type_tab.type_int.clone()),
        Type::Atom(type_tab.type_float.clone()),
        Type::Atom(type_tab.type_string.clone()),
        Type::Atom(type_tab.type_object.clone())
    ], clauses: vec![
        (type_tab.list_of(Type::Var(TypeId(VAR0))),
        Condition::Bound(Type::Var(TypeId(VAR0)),
            Bound::Trait(trait_tab.trait_eq.clone())))
    ]});
    PredicateTable {map}
}

fn type_eq(t1: &Type, t2: &Type) -> bool {
    match t1 {
        Type::Atom(t1) => match t2 {
            Type::Atom(t2) => Rc::ptr_eq(t1,t2),
            _ => false
        },
        _ => false
    }
}

impl PredicateTable {
    pub(super) fn new(type_table: &TypeTable, trait_table: &TraitTable) -> Self {
        init_table(type_table, trait_table)
    }
    fn apply(&self, trait_id: &TraitId, typ: &Type) -> bool {
        let rules = match self.map.get(trait_id) {
            Some(value) => value,
            None => unreachable!("Trait does not exist: {}", trait_id)
        };
        for ty in &rules.leaf {
            if type_eq(typ, ty) {return true;}
        }
        let mut subs = Substitution::new();
        let none = &mut None;
        for (ty, cond) in &rules.clauses {
            if let Ok(()) = subs.unify(ty, typ, none) {
                return self.check_clause(subs, cond);
            } else {
                subs.map.clear();
            }
        }
        false
    }
    pub fn apply_bound(&self, bound: &Bound, typ: &Type) -> bool {
        match bound {
            Bound::None => true,
            Bound::Trait(trait_id) => self.apply(trait_id, typ),
            Bound::Union(list) =>  list.iter()
                .all(|trait_id| self.apply(trait_id, typ))
        }
    }
    fn check_clause(&self, subs: Substitution, cond: &Condition) -> bool {
        match cond {
            Condition::None => true,
            Condition::Bound(typ, pred) => {
                let typ = subs.apply(typ);
                if let Type::Var(_) = typ {return false;}
                self.apply_bound(pred, &typ)
            }
        }
    }
    pub fn extend(&mut self, trait_id: &TraitId, typ: Type) {
        let rules = match self.map.get_mut(trait_id) {
            Some(value) => value,
            None => unreachable!("Trait does not exist: {}", trait_id)
        };
        rules.leaf.push(typ);
    }
    pub fn extend_bound(&mut self, bound: &Bound, typ: Type) {
        match bound {
            Bound::None => {},
            Bound::Trait(trait_id) => self.extend(trait_id, typ),
            Bound::Union(list) => {
                for trait_id in list.iter() {
                    self.extend(trait_id, typ.clone())
                }
            }
        }
    }
}
