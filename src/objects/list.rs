
use std::rc::Rc;
use std::cell::RefCell;

use crate::object::{
    Object, FnResult, Function, List,
    VARIADIC, MutableFn,
};
use crate::vm::Env;
use crate::rand::Rand;
use crate::global::list;
use crate::class::Class;

fn push(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match *pself {
        Object::List(ref a) => {
            match a.try_borrow_mut() {
                Ok(mut a) => {
                    if a.frozen {
                        return env.value_error("Value error in a.push(): a is frozen.");
                    }
                    for x in argv {
                        a.v.push(x.clone());
                    }
                    Ok(Object::Null)
                },
                Err(_) => {env.std_exception(
                    "Memory error in a.push(x): internal buffer of a was aliased.\n\
                     Try to replace a by copy(a) at some place."
                )}
            }
        },
        ref a => env.type_error1("Type error in a.push(x): a is not a list.","a",a)
    }
}

fn plus(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    push(env, pself, argv)?;
    Ok(pself.clone())
}

fn append(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
    match *pself {
        Object::List(ref a) => {
            if a.borrow_mut().frozen {
                return env.value_error("Value error in a.append(): a is frozen.");
            }
            for obj in argv {
                match obj {
                    Object::List(ref ai) => {
                        let mut v = (&ai.borrow().v[..]).to_vec();
                        let mut a = a.borrow_mut();
                        a.v.append(&mut v);
                    },
                    ref b => return env.type_error1(
                        "Type error in a.append(b): b is not a list.",
                        "b",b)
                }
            }
            Ok(Object::Null)
        },
        _ => env.type_error("Type error in a.append(b): a is not a list.")
    }
}

fn pop_at_index(env: &mut Env, v: &mut Vec<Object>, index: &Object)
-> FnResult
{
    let len = v.len();
    let i = match *index {
        Object::Int(i) => if i>=0 {i as usize} else {
            // #overflow-transmutation: len as isize
            let i = i as isize + len as isize;
            if i>=0 {i as usize} else {
                return env.index_error(
                    "Index error in a.pop(i): i is out of lower bound.");
            }
        },
        ref i => return env.type_error1(
            "Type error in a.pop(i): is not an integer.","i",i)
    };
    if i<len {
        Ok(v.remove(i))
    } else {
        env.index_error(
            "Index error in a.pop(i): i is is out of upper bound.")
    }
}

fn pop(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match *pself {
        Object::List(ref a) => {
            match a.try_borrow_mut() {
                Ok(mut a) => {
                    if a.frozen {
                        return env.value_error("Value error in a.pop(): a is frozen.");
                    }
                    if !argv.is_empty() {
                        pop_at_index(env, &mut a.v, &argv[0])
                    } else {
                        match a.v.pop() {
                            Some(x) => Ok(x),
                            None => env.value_error(
                                "Value error in a.pop(): a is empty.")
                        }
                    }
                },
                Err(_) => env.std_exception(
                    "Memory error in a.pop(): internal buffer of a is aliased.\n\
                     Try to replace a by copy(a) at some place.")
            }
        },
        ref a => env.type_error1("Type error in a.pop(): a is not a list.","a",a)
    }
}

fn insert(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
    match argv.len() {
        2 => {}, n => return env.argc_error(n,0,0,"insert")
    }
    match *pself {
        Object::List(ref a) => {
            match a.try_borrow_mut() {
                Ok(mut a) => {
                    let index = match argv[0] {
                        Object::Int(i) => if i<0 {
                            let i = i as isize+a.v.len() as isize;
                            if i<0 {
                                return env.index_error("Index error in a.insert(i,x): i is out of lower bound.");
                            } else {i as usize}
                        } else {i as usize},
                        ref i => return env.type_error1("Type error in a.insert(i,x): i is not an integer.","i",i)
                    };
                    if a.frozen {
                        return env.value_error("Value error in a.pop(): a is frozen.");
                    }
                    if index < a.v.len() {
                        a.v.insert(index,argv[1].clone());
                    } else {
                        return env.index_error("Index error in a.insert(i,x): i is out of upper bound.");
                    }
                    Ok(Object::Null)
                },
                Err(_) => {
                    env.std_exception("Memory error in a.insert(i,x): internal buffer of a is aliased.")
                }
            }
        },
        ref a => env.type_error1(
            "Type error in a.insert(i,x): a is not a list.",
            "a", a)
    }
}

fn size(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"size")
    }
    match *pself {
        Object::List(ref a) => {
            Ok(Object::Int(a.borrow().v.len() as i32))
        },
        _ => env.type_error("Type error in a.size(): a is not a list.")
    }
}

fn map(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    if argv.len() != 1 {
        return env.argc_error(argv.len(),1,1,"map");
    }
    let a = match *pself {
        Object::List(ref a) => a.borrow(),
        _ => return env.type_error("Type error in a.map(f): a is not a list.")
    };
    let mut acc: Vec<Object> = Vec::with_capacity(a.v.len());
    for x in &a.v {
        let y = env.call(&argv[0],&Object::Null,&[x.clone()])?;
        acc.push(y);
    }
    Ok(List::new_object(acc))
}

fn filter(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    if argv.len() != 1 {
        return env.argc_error(argv.len(),1,1,"filter");
    }
    let a = match *pself {
        Object::List(ref a) => a.borrow(),
        _ => return env.type_error("Type error in a.filter(p): a is not a list.")
    };
    let mut acc: Vec<Object> = Vec::new();
    for x in &a.v {
        let y = env.call(&argv[0], &Object::Null, &[x.clone()])?;
        let condition = match y {
            Object::Bool(u) => u,
            ref value => return env.type_error2(
                "Type error in a.filter(p): return value of p is not of boolean type.",
                "x", "p(x)", x, value)
        };
        if condition {
            acc.push(x.clone());
        }
    }
    Ok(List::new_object(acc))
}

fn new_shuffle() -> MutableFn {
    let mut rng = Rand::new(0);
    Box::new(move |env: &mut Env, pself: &Object, argv: &[Object]| -> FnResult {
        match argv.len() {
            0 => {}, n => return env.argc_error(n,0,0,"shuffle")
        }
        match *pself {
            Object::List(ref a) => {
                let mut ba = a.borrow_mut();
                rng.shuffle(&mut ba.v);
                Ok(Object::List(a.clone()))
            },
            _ => env.type_error("Type error in a.shuffle(): a is not a list.")
        }
    })
}

fn list_chain(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    let infix = match argv.len() {
        0 => None,
        1 => Some(&argv[0]),
        n => return env.argc_error(n,0,0,"chain")
    };
    let a = match *pself {
        Object::List(ref a) => a.borrow(),
        _ => return env.type_error("Type error in a.chain(): a is not a list.")
    };
    let mut acc: Vec<Object> = Vec::new();
    let mut first = true;
    for t in &a.v {
        if let Some(infix) = infix {
            if first {first = false} else {acc.push(infix.clone());}
        }
        match *t {
            Object::List(ref t) => {
                for x in &t.borrow().v {
                    acc.push(x.clone());
                }
            },
            ref x => {
                acc.push(x.clone());
            }
        }
    }
    Ok(List::new_object(acc))
}

fn list_rev(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    let mut a = match *pself {
        Object::List(ref a) => a.borrow_mut(),
        _ => return env.type_error("Type error in a.rev(): a is not a list.")
    };
    match argv.len() {
        0 => {
            a.v[..].reverse();
            Ok(pself.clone())
        },
        n => env.argc_error(n,0,0,"rev")
    }
}

fn list_swap(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    let mut a = match *pself {
        Object::List(ref a) => a.borrow_mut(),
        _ => return env.type_error("Type error in a.swap(i,j): a is not a list.")
    };
    match argv.len() {
        2 => {
            let x = match argv[0] {
                Object::Int(x)=>x,
                ref index => return env.type_error1(
                    "Type error in a.swap(i,j): i is not an integer.",
                    "i",index)
            };
            let y = match argv[1] {
                Object::Int(y)=>y,
                ref index => return env.type_error1(
                    "Type error in a.swap(i,j): j is not an integer.",
                    "j",index)
            };
            let len = a.v.len();
            let i = if x<0 {
                // #overflow-transmutation: len as isize
                let x = x as isize + len as isize;
                if x<0 {
                    return env.index_error("Index error in a.swap(i,j): i is out of lower bound.");
                } else {
                    x as usize
                }
            } else if x as usize >= len {
                return env.index_error("Index error in a.swap(i,j): i is out of upper bound.");
            } else {
                x as usize
            };
            let j = if y<0 {
                // #overflow-transmutation: len as isize
                let y = y as isize + len as isize;
                if y<0 {
                    return env.index_error("Index error in a.swap(i,j): j is out of lower bound.");
                } else {
                    y as usize
                }
            } else if y as usize >= len {
                return env.index_error("Index error in a.swap(i,j): j is out of upper bound.");
            } else {
                y as usize
            };
            a.v.swap(i,j);
            Ok(Object::Null)
        },
        n => env.argc_error(n,2,2,"swap")
    }
}

pub fn duplicate(a: &Rc<RefCell<List>>, n: usize) -> Object {
    let a = a.borrow();
    let size = a.v.len()*n;
    let mut acc: Vec<Object> = Vec::with_capacity(size);
    for _ in 0..n {
        for x in &a.v {
            acc.push(x.clone());
        }
    }
    List::new_object(acc)
}

pub fn map_fn(env: &mut Env, f: &Object, argv: &[Object]) -> FnResult {
    let argc = argv.len();
    if argc == 0 {
        return Ok(List::new_object(Vec::new()));
    }
    let mut v: Vec<Rc<RefCell<List>>> = Vec::with_capacity(argc);
    for obj in argv {
        match obj {
            Object::List(ref a) => v.push(a.clone()),
            ref a => {
                let y = list(env,a)?;
                // todo: traceback
                v.push(match y {Object::List(a) => a, _ => unreachable!()});
            }
        }
    }
    let n = v[0].borrow().v.len();
    for obj in &v {
        if n != obj.borrow().v.len() {
            return env.type_error("Type error in f[a1,...,an]: all lists must have the same size.");
        }
    }

    let null = &Object::Null;
    let mut acc: Vec<Object> = Vec::with_capacity(argc);
    let mut args: Vec<Object> = vec![Object::Null; argc];
    for k in 0..n {
        for i in 0..argc {
            args[i] = v[i].borrow().v[k].clone();
        }
        let y = env.call(f, null, &args)?;
        acc.push(y);
    }
    Ok(List::new_object(acc))
}

fn clear(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
    match *pself {
        Object::List(ref a) => {
            match a.try_borrow_mut() {
                Ok(mut a) => {
                    if a.frozen {
                        return env.value_error("Value error in a.clear(): a is frozen.");
                    }
                    match argv.len() {
                        0 => {a.v.clear();},
                        1 => {
                            let n = match argv[0] {
                                Object::Int(n) => if n<0 {0} else {n as usize},
                                _ => return env.type_error("Type error in a.clear(n): n is not an integer.")
                            };
                            a.v.truncate(n);
                        },
                        n => return env.argc_error(n,0,1,"clear")
                    }
                    Ok(Object::Null)
                },
                Err(_) => env.std_exception(
                    "Memory error in a.clear(x): internal buffer of a was aliased.")
            }
        },
        ref a => env.type_error1(
            "Type error in a.clear(): a is not a list.",
            "a", a)
    }
}

pub fn cartesian_product(a: &List, b: &List) -> Object {
    let mut acc: Vec<Object> = Vec::with_capacity(a.v.len()*b.v.len());
    for x in &a.v {
        for y in &b.v {
            acc.push(List::new_object(vec![x.clone(), y.clone()]));
        }
    }
    List::new_object(acc)
}

pub fn cartesian_power(v: &[Object], n: i32) -> Object {
    let n = if n < 0 {0} else {n as u32};
    let m = v.len();
    let len = m.pow(n);
    let mut y: Vec<Vec<Object>> = Vec::with_capacity(len);
    if m == 0 {
        let y = List::new_object(Vec::new());
        if n == 0 {
            return List::new_object(vec![y]);
        } else {
            return y;
        }
    }
    for _ in 0..len {
        y.push(Vec::new());
    }
    let mut k = len/m;
    let mut count = 1;
    for _ in 0..n {
        let mut j = 0;
        for _ in 0..count {
            for obj in &v[0..m] {
                for _ in 0..k {
                    y[j].push(obj.clone());
                    j += 1;
                }
            }
        }
        count *= m;
        k /= m;
    }
    List::new_object(y.into_iter()
       .map(List::new_object).collect())
}

fn rotate(a: &mut [Object], n: i32) {
    let m = a.len();
    if n >= 0 {
        let mut n = n as usize;
        if n >= m {n %= m;}
        a.rotate_right(n);
    } else {
        let mut n = (-n) as usize;
        if n >= m {n %= m;}
        a.rotate_left(n);
    }
}

fn list_rot(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"rot")
    }
    match *pself {
        Object::List(ref a) => {
            let n = match argv[0] {
                Object::Int(n) => n,
                ref n => return env.type_error1(
                    "Type error in a.rot(n): n is not an integer.","n",n)
            };
            match a.try_borrow_mut() {
                Ok(mut a) => {
                    if a.frozen {
                        return env.value_error("Value error in a.clear(): a is frozen.");
                    }
                    rotate(&mut a.v,n);
                    Ok(pself.clone())
                },
                Err(_) => {env.std_exception(
                    "Memory error in a.rot(n): internal buffer of a was aliased."
                )}
            }
        },
        ref x => env.type_error1(
            "Type error in a.rot(n): a is not a list","a",x)
    }
}

pub fn init(t: &Class){
    let mut m = t.map.borrow_mut();
    m.insert_fn_plain("push", push, 0, VARIADIC);
    m.insert_fn_plain("plus", plus, 0, VARIADIC);
    m.insert_fn_plain("append", append, 0, VARIADIC);
    m.insert_fn_plain("pop", pop, 0, 0);
    m.insert_fn_plain("insert", insert, 2, 2);
    m.insert_fn_plain("size", size, 0, 0);
    m.insert_fn_plain("map", map, 1, 1);
    m.insert_fn_plain("filter", filter, 1, 1);
    m.insert_fn_plain("chain", list_chain, 0, 0);
    m.insert_fn_plain("rev", list_rev, 0, 0);
    m.insert_fn_plain("swap", list_swap, 2, 2);
    m.insert_fn_plain("clear", clear, 0, 1);
    m.insert_fn_plain("rot", list_rot, 1, 1);
    m.insert("shuffle", Function::mutable(new_shuffle(), 0, 0));
}
