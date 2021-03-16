
use crate::object::{
    Object, FnResult, List, Map,
    VARIADIC
};
use crate::vm::Env;
use crate::iterable::new_iterator;
use crate::class::Class;

pub fn map_update(m: &mut Map, m2: &Map){
    for (key,value) in &m2.m {
        m.m.insert(key.clone(), value.clone());
    }
}

pub fn map_extend(m: &mut Map, m2: &Map) {
    for (key,value) in &m2.m {
        if !m.m.contains_key(key) {
            m.m.insert(key.clone(), value.clone());
        }
    }
}

fn update(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"update")
    }
    match *pself {
        Object::Map(ref m) => {
            match argv[0] {
                Object::Map(ref m2) => {
                    let m = &mut *m.borrow_mut();
                    if m.frozen {
                        return env.value_error("Value error in m.update(m2): m is frozen.");
                    }
                    map_update(m,&*m2.borrow());
                    Ok(Object::Null)
                },
                _ => env.type_error("Type error in m.update(m2): m2 is not a map.")
            }
        },
        _ => env.type_error("Type error in m.update(m2): m is not a map.")
    }
}

fn extend(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"extend")
    }
    match *pself {
        Object::Map(ref m) => {
            match argv[0] {
                Object::Map(ref m2) => {
                    let m = &mut *m.borrow_mut();
                    if m.frozen {
                        return env.value_error("Value error in m.extend(m2): m is frozen.");
                    }
                    map_extend(m,&*m2.borrow());
                    Ok(Object::Null)
                },
                _ => env.type_error("Type error in m.extend(m2): m2 is not a map.")
            }
        },
        _ => env.type_error("Type error in m.extend(m2): m is not a map.")
    }
}

fn values(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"values")
    }
    if let Object::Map(ref m) = *pself {
        let mut index: usize = 0;
        let v: Vec<Object> = m.borrow().m.values().cloned().collect();
        let f = Box::new(move |_env: &mut Env, _pself: &Object, _argv: &[Object]| -> FnResult {
            Ok(if index == v.len() {
                Object::empty()
            } else {
                index += 1;
                v[index-1].clone()
            })
        });
        Ok(new_iterator(f))
    } else {
        env.type_error("Type error in m.values(): m is not a map.")
    }
}

fn items(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"items")
    }
    if let Object::Map(ref m) = *pself {
        let mut index: usize = 0;
        let m = &m.borrow().m;
        let mut keys: Vec<Object> = Vec::with_capacity(m.len());
        let mut values: Vec<Object> = Vec::with_capacity(m.len());
        for (key,value) in m.iter() {
            keys.push(key.clone());
            values.push(value.clone());
        }
        let f = Box::new(move |_env: &mut Env, _pself: &Object, _argv: &[Object]| -> FnResult {
            Ok(if index == keys.len() {
                Object::empty()
            } else {
                index += 1;
                let t = vec![keys[index-1].clone(), values[index-1].clone()];
                List::new_object(t)
            })
        });
        Ok(new_iterator(f))
    } else {
        env.type_error("Type error in m.items(): m is not a map.")
    }
}

fn clear(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"clear")
    }
    match *pself {
        Object::Map(ref m) => {
            let mut m = m.borrow_mut();
            if m.frozen {
                return env.value_error("Value error in m.clear(): m is frozen.");
            }
            m.m.clear();
            Ok(Object::Null)
        },
        _ => env.type_error("Type error in m.clear(): m is not a map.")
    }
}

fn remove(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"remove")
    }
    match *pself {
        Object::Map(ref m) => {
            let mut m = m.borrow_mut();
            if m.frozen {
                return env.value_error("Value error in m.remove(key): m is frozen.");
            }
            match m.m.remove(&argv[0]) {
                Some(value) => Ok(value),
                None => env.index_error("Index error in m.remove(key): key was not in m.")
            }
        },
        _ => env.type_error("Type error in m.remove(key): m is not a map.")
    }
}

fn add(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match *pself {
        Object::Map(ref m) => {
            let mut m = m.borrow_mut();
            if m.frozen {
                return env.value_error("Value error in m.add(key): m is frozen.");
            }
            for x in argv {
                m.m.insert(x.clone(),Object::Null);
            }
            Ok(Object::Null)
        },
        ref m => env.type_error1("Type error in m.add(key): m is not a map.","m",m)
    }
}

pub fn subseteq(a: &Map, b: &Map) -> bool {
    let bm = &b.m;
    for key in a.m.keys() {
        if !bm.contains_key(key) {return false;}
    }
    true
}

pub fn subset(a: &Map, b: &Map) -> bool {
    a.m.len() < b.m.len() && subseteq(a,b)
}

pub fn init(t: &Class){
    let mut m = t.map.borrow_mut();
    m.insert_fn_plain("update", update, 1, 1);
    m.insert_fn_plain("extend", extend, 1, 1);
    m.insert_fn_plain("values", values, 0, 0);
    m.insert_fn_plain("items", items, 0, 0);
    m.insert_fn_plain("clear", clear, 0, 0);
    m.insert_fn_plain("remove", remove, 0, 0);
    m.insert_fn_plain("add", add, 0, VARIADIC);
}
