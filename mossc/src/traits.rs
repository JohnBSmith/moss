
use std::rc::Rc;
use std::collections::HashMap;

use crate::typing::{Type, TypeTable, TraitTable, Bound};

struct Rules {
    leaf: Vec<Type>
}

type TraitId = Rc<str>;

pub struct PredicateTable {
    map: HashMap<TraitId, Rules>
}

fn init_table(type_tab: &TypeTable, trait_tab: &TraitTable)
-> PredicateTable
{
    let mut map = HashMap::new();
    map.insert(trait_tab.trait_add.clone(), Rules {leaf: vec![
        Type::Atom(type_tab.type_int.clone()),
        Type::Atom(type_tab.type_float.clone()),
        Type::Atom(type_tab.type_string.clone()),
        Type::Atom(type_tab.type_object.clone())
    ]});
    map.insert(trait_tab.trait_sub.clone(), Rules {leaf: vec![
        Type::Atom(type_tab.type_int.clone()),
        Type::Atom(type_tab.type_float.clone()),
        Type::Atom(type_tab.type_object.clone())
    ]});
    map.insert(trait_tab.trait_mul.clone(), Rules {leaf: vec![
        Type::Atom(type_tab.type_int.clone()),
        Type::Atom(type_tab.type_float.clone()),
        Type::Atom(type_tab.type_object.clone())
    ]});
    map.insert(trait_tab.trait_div.clone(), Rules {leaf: vec![
        Type::Atom(type_tab.type_float.clone()),
        Type::Atom(type_tab.type_object.clone())
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
    pub fn apply(&self, trait_id: &TraitId, typ: &Type) -> bool {
        let rules = match self.map.get(trait_id) {
            Some(value) => value,
            None => unreachable!("Trait does not exist: {}", trait_id)
        };
        for ty in &rules.leaf {
            if type_eq(typ, ty) {return true;}
        }
        false
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
