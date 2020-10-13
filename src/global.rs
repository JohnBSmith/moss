
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::char;
use std::str::FromStr;
use std::f64::NAN;
use std::io::Read;

use crate::vm::{
    RTE, Env, op_lt, interface_index, interface_types_set
};
use crate::object::{
    Object, Table, Map, List, CharString,
    FnResult, Function, EnumFunction, Info,
    VARIADIC, float, new_module, downcast
};
use crate::rand::Rand;
use crate::iterable::{iter,cycle};
use crate::system::{History,open_module_file};
use crate::module::eval_module;
use crate::compiler::Value;
use crate::long::{Long, pow_mod};
use crate::class::{table_get};
use crate::iterable::new_iterator;
use crate::map::map_extend;
use crate::class::{Class,class_new};
use crate::range::Range;
use crate::data::{Bytes,base16};

pub fn type_name(env: &mut Env, x: &Object) -> String {
    return match *x {
        Object::Null => "null",
        Object::Bool(_) => "Bool",
        Object::Int(_) => "Int",
        Object::Float(_) => "Float",
        Object::Complex(_) => "Complex",
        Object::List(_) => "List",
        Object::String(_) => "String",
        Object::Map(_) => "Map",
        Object::Function(_) => "Function",
        Object::Info(x) => {
            match x {
                Info::Empty => "Empty",
                Info::Unimplemented => "Unimplemented"
            }
        },
        Object::Interface(ref x) => return x.type_name(env)
    }.to_string();
}

pub fn fpanic(_env: &mut Env, _pself: &Object, _argv: &[Object]) -> FnResult{
    panic!()
}

pub fn print(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult{
    for i in 0..argv.len() {
        print!("{}",argv[i].string(env)?);
    }
    println!();
    return Ok(Object::Null);
}

pub fn put(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult{
    for i in 0..argv.len() {
        print!("{}",argv[i].string(env)?);
    }
    return Ok(Object::Null);
}

fn float_to_string(env: &Env, x: &Object, fmt: &Object, precision: &Object) -> FnResult {
    let n = match *precision {
        Object::Int(n) => if n<0 {0} else {n as usize},
        _ => return env.type_error("Type error in str(x,fmt,precision): precision is not an integer.")
    };
    let fmt = match *fmt {
        Object::String(ref s) => &s.data,
        _ => return env.type_error("Type error in str(x,fmt,precision): fmt is not a string.")
    };
    let x = match *x {
        Object::Int(n) => float(n),
        Object::Float(f) => f,
        _ => return env.type_error("Type error in str(x,fmt,precision): x should be of type Float or Int.")
    };
    if fmt.len() != 1 {
        return env.value_error("Value error in str(x,fmt,precision): size(fmt)!=1.");
    }
    let s = match fmt[0] {
        'f' => {format!("{:.*}",n,x)},  // fixed point
        'e' => {format!("{:.*e}",n,x)}, // lower exponential
        'E' => {format!("{:.*E}",n,x)}, // upper exponential
        't' => { // fixed point, trimmed zeroes
            let mut v: Vec<char> = format!("{:.*}",n,x).chars().collect();
            loop{
                let n = v.len();
                if n<2 {break;}
                if v[n-1]=='0' || v[n-1]=='.' {
                    v.pop();
                }else{
                    break;
                }
            }
            return Ok(CharString::new_object(v));
        },
        _ => {
            return env.value_error("Value error in str(x,fmt,precision): fmt should be one of 'f', 'e', 'E'.");
        }
    };
    return Ok(CharString::new_object_str(&s));
}

fn fstr(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult{
    match argv.len() {
        1 => {
            let s = argv[0].string(env)?;
            return Ok(CharString::new_object_str(&s));
        },
        3 => {
            return float_to_string(env,&argv[0],&argv[1],&argv[2]);
        },
        n => {
            return env.argc_error(n,1,1,"str");
        }
    }
}

fn repr(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"repr")
    }
    let s = argv[0].repr(env)?;
    return Ok(CharString::new_object_str(&s));
}

fn sgn(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"sgn")
    }
    match argv[0] {
        Object::Int(x) => {
            return Ok(Object::Int(if x<0 {-1} else if x==0 {0} else {1}));
        },
        Object::Float(x) => {
            return Ok(Object::Float(x.signum()));
        },
        Object::Complex(z) => {
            return Ok(Object::Complex(z/z.abs()));
        },
        Object::Interface(ref x) => {
            return x.clone().sgn(env);
        },
        _ => {
            return env.type_error1(
                "Type error in sgn(x): x should be of type Int, Long, Float.",
                "x",&argv[0]
            );
        }
    }
}

fn abs(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"abs")
    }
    'type_error: loop{
    match argv[0] {
        Object::Int(x) => {
            return Ok(Object::Int(x.abs()));
        },
        Object::Float(x) => {
            return Ok(Object::Float(x.abs()));
        },
        Object::Complex(z) => {
            return Ok(Object::Float(z.abs()));
        },
        Object::Interface(ref x) => {
            return x.clone().abs(env);
        },
        _ => break 'type_error
    }
    } // type_error:
    return env.type_error1(
        "Type error in abs(x): x should be of type Int, Long, Float, Complex.",
        "x",&argv[0]
    );
}

fn eval(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult{
    let gtab = match argv.len() {
        1 => env.rte().pgtab.borrow().clone(),
        2 => {
            match argv[1] {
                Object::Map(ref m) => m.clone(),
                _ => return env.type_error2(
                    "Type error in eval(s,m): m is not a map.",
                    "s","m",&argv[0],&argv[1])
            }
        },
        n => {
            return env.argc_error(n,1,1,"eval")
        }
    };
    match argv[0] {
        Object::String(ref s) => {
            let s = s.to_string();
            return match env.eval_string(&s,"eval",gtab,Value::Optional) {
                Ok(x) => {Ok(x)},
                Err(e) => Err(e)
            }
        },
        _ => {
            return env.type_error1(
                "Type error in eval(s): s is not a string.",
                "s", &argv[0]
            );
        }
    }
}

fn range_length(a: i32, b: i32, step: i32) -> Option<i32> {
    if step>0 && b>=a {
        return Some((b-a)/step+1);
    }else if step<0 && b<=a {
        return Some((a-b)/(-step)+1);
    }else if step != 0 {
        return Some(0);
    }else{
        return None;
    }
}

fn len(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"len")
    }
    match argv[0] {
        Object::List(ref a) => {
            Ok(Object::Int(a.borrow().v.len() as i32))
        },
        Object::Map(ref m) => {
            Ok(Object::Int(m.borrow().m.len() as i32))
        },
        Object::String(ref s) => {
            Ok(Object::Int(s.data.len() as i32))
        },
        Object::Interface(ref x) => {
            if let Some(r) = (*x).as_any().downcast_ref::<Range>() {
                if let Object::Int(a) = r.a {
                if let Object::Int(b) = r.b {
                if let Object::Null = r.step {
                    return Ok(Object::Int(b-a+1));
                }else if let Object::Int(step) = r.step {
                    if let Some(value) = range_length(a,b,step) {
                        return Ok(Object::Int(value));
                    }else{
                        return env.value_error(
                            "Value error in len(a..b: step): step==0.")
                    }
                }}}
            }
            let f = x.clone().get(&Object::from("len"),env)?;
            return env.call(&f,&argv[0],&[]);
        },
        _ => env.type_error1(
            "Type error in len(a): cannot determine the length of a.",
            "a", &argv[0]
        )
    }
}

fn load_file(env: &mut Env, id: &str) -> FnResult {
    if !env.rte().read_access(id) {
        return env.std_exception(&format!(
            "Error in load(id): Could not open file id=='{}': permission denied.",id
        ));
    }
    let path = env.rte().path.clone();
    let search_paths = &path.borrow().v;
    let (mut f,binary) = match open_module_file(search_paths,id) {
        Ok(value) => value,
        Err(e) => return env.std_exception(&e)
    };

    let module = new_module(id);
    env.rte().clear_at_exit(module.map.clone());
    let cond = env.rte().main_module.get();
    let value = if binary {
        env.rte().main_module.set(false);
        eval_module(env,module.map.clone(),&mut f,id)
    }else{
        let mut s = String::new();
        if f.read_to_string(&mut s).is_err() {
            return env.std_exception(&format!(
                "Error in load: could not read file '{}.moss'.",id));
        }
        env.rte().main_module.set(false);
        env.eval_string(&s,id,module.map.clone(),Value::None)
    };
    env.rte().main_module.set(cond);
    return Ok(match value? {
        Object::Null => Object::Interface(Rc::new(module)),
        x => x
    });
}

fn load(env: &mut Env, id: Rc<CharString>, hot_plug: bool) -> FnResult{
    if !hot_plug {
        let m = env.rte().module_table.borrow();
        if let Some(value) = m.m.get(&Object::String(id.clone())) {
            return Ok(value.clone());
        }
    }
    let s = id.to_string();
    let y = match &s[..] {
        "fs" => crate::fs::load_fs(env),

        #[cfg(feature = "la")]
        "la" => crate::la::load_la(env),

        "math"  => crate::math::load_math(),

        #[cfg(feature = "math-la")]
        "math/la" => crate::math_la::load_math_la(env),

        #[cfg(feature = "math-sf")]
        "math/sf"    => crate::sf::load_sf(),

        #[cfg(feature = "math-sf")]
        "math/sf/ei" => crate::sf::load_sf_ei(),

        "cmath" => crate::math::load_cmath(),
        "regex" => crate::regex::load_regex(env),
        "sys"   => crate::sys::load_sys(env.rte()),
        "sysfn" => crate::sysfn::load_sysfn(env.rte()),
        "time"  => crate::time::load_time(),
        "data" => crate::data::load_data(env),
        
        
        #[cfg(feature = "graphics")]
        "graphics" => crate::graphics::load_graphics(),

        _ => {
            load_file(env,&s)?
            // return index_error(&format!("Could not load module '{}'.",s));
        }
    };
    if !hot_plug {
        let mut m = env.rte().module_table.borrow_mut();
        m.m.insert(Object::String(id),y.clone());
    }
    return Ok(y);
}

fn fload(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"load")
    }
    match argv[0] {
        Object::String(ref s) => load(env,s.clone(),false),
        _ => env.type_error1(
            "Type error in load(id): id is not a string.",
            "id", &argv[0]
        )
    }
}

fn fiter(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"iter")
    }
    return iter(env,&argv[0]);
}

fn record(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"record")
    }
    if let Some(t) = downcast::<Table>(&argv[0]) {
        Ok(Object::Map(t.map.clone()))
    }else if let Some(t) = downcast::<Table>(&argv[0]) {
        Ok(Object::Map(t.map.clone()))
    }else if let Some(t) = downcast::<Class>(&argv[0]) {
        Ok(Object::Map(t.map.clone()))
    }else{
        env.type_error1(
            "Type error in record(x): x is not a table.",
            "x", &argv[0]
        )
    }
}

fn fobject(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        0 => {
            Ok(Object::Interface(Table::new(Object::Null)))
        },
        1 => {
            Ok(Object::Interface(Table::new(argv[0].clone())))
        },
        2 => {
            match argv[1] {
                Object::Map(ref m) => {
                    Ok(Object::Interface(Rc::new(Table{
                        prototype: argv[0].clone(),
                        map: m.clone()
                    })))
                },
                _ => env.type_error1(
                    "Type error in object(p,m): m is not a map.",
                    "m", &argv[1]
                )
            }
        },
        n => env.argc_error(n,0,0,"object")
    }
}

fn ftype(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"type")
    }
    return Ok(match argv[0] {
        Object::Null => Object::Null,
        Object::Bool(_) => Object::Interface(env.rte().type_bool.clone()),
        Object::Int(_) => Object::Interface(env.rte().type_int.clone()),
        Object::Float(_) => Object::Interface(env.rte().type_float.clone()),
        Object::Complex(_) => Object::Interface(env.rte().type_complex.clone()),
        Object::String(_) => Object::Interface(env.rte().type_string.clone()),
        Object::List(_) => Object::Interface(env.rte().type_list.clone()),
        Object::Map(_) => Object::Interface(env.rte().type_map.clone()),
        Object::Function(_) => Object::Interface(env.rte().type_function.clone()),
        Object::Interface(ref x) => return x.get_type(env),
        _ => Object::Null
    });
}

fn float_range_to_list(env: &mut Env, r: &Range) -> FnResult {
    let a = match r.a {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        _ => return env.type_error1(
            "Type error in list(a..b): a is not of type Float.",
            "a",&r.a)
    };
    let b = match r.b {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        _ => return env.type_error1(
            "Type error in list(a..b): b is not of type Float.",
            "b",&r.b
        )
    };
    let d = match r.step {
        Object::Null => 1.0,
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        _ => return env.type_error1(
            "Type error in list(a..b: d): d is not of type Float.",
            "d",&r.step
        )
    };

    let q = (b-a)/d;
    let n = if q<0.0 {0} else {(q+0.001) as usize+1};

    let mut v: Vec<Object> = Vec::with_capacity(n);
    for k in 0..n {
        v.push(Object::Float(a+float(k)*d));
    }
    return Ok(List::new_object(v));
}

pub fn list(env: &mut Env, obj: &Object) -> FnResult {
    match *obj {
        Object::Int(n) => {
            if n<0 {
                return env.value_error("Value error in list(n): n<0.");
            }
            let mut v: Vec<Object> = Vec::with_capacity(n as usize);
            for i in 0..n {
                v.push(Object::Int(i));
            }
            return Ok(List::new_object(v));
        },
        Object::List(ref a) => {
            return Ok(Object::List(a.clone()));
        },
        Object::Map(ref m) => {
            let v: Vec<Object> = m.borrow().m.keys().cloned().collect();
            return Ok(List::new_object(v));
        },
        Object::String(ref s) => {
            let mut v: Vec<Object> = Vec::with_capacity(s.data.len());
            for x in &s.data {
                v.push(CharString::new_object_char(*x));
            }
            return Ok(List::new_object(v));
        },
        Object::Function(_) => {
            return crate::iterable::to_list(env,obj,&[]);
        },
        _ => {}
    }

    if let Some(r) = downcast::<Range>(obj) {
        let a = match r.a {
            Object::Int(x)=>x,
            Object::Float(_) => {
                return float_range_to_list(env,r);
            },
            _ => {
                return crate::iterable::to_list(env,obj,&[])
            }
        };
        let b = match r.b {
            Object::Int(x)=>x,
            Object::Float(_) => {
                return float_range_to_list(env,r);
            },
            _ => return env.type_error1(
                "Type error in list(a..b): b is not an integer.",
                "b",&r.b)
        };
        let d = match r.step {
            Object::Null => 1,
            Object::Int(x)=>x,
            Object::Float(_) => {
                return float_range_to_list(env,r);
            },
            _ => return env.type_error1(
                "Type error in list(a..b: d): d is not an integer.",
                "d",&r.step)
        };
        if d==0 {
            return env.value_error("Value error in list(a..b: d): d==0.");
        }
        let mut n = (b-a)/d+1;
        if n<0 {n=0;}
        let mut v: Vec<Object> = Vec::with_capacity(n as usize);
        let mut k = a;
        for _ in 0..n {
            v.push(Object::Int(k));
            k+=d;
        }
        return Ok(List::new_object(v));
    }
    
    if let Object::Interface(x) = obj {
        let key = env.rte().key_list.clone();
        let f = x.clone().get(&key,env)?;
        return env.call(&f,obj,&[]);
    }

    return env.type_error1(
        "Type error in list(x): cannot convert x into a list.",
        "x", obj
    );
}

pub fn flist(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult{
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"list")
    }
    return list(env,&argv[0]);
}

fn set(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"set")
    }
    let i = &iter(env,&argv[0])?;
    let mut m: HashMap<Object,Object> = HashMap::new();
    loop {
        let y = env.call(i,&Object::Null,&[])?;
        if y.is_empty() {break;}
        m.insert(y,Object::Null);
    }
    return Ok(Map::new_object(m));
}

fn copy(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"copy")
    }
    match argv[0] {
        Object::List(ref a) => {
            Ok(List::new_object(a.borrow().v.clone()))
        },
        Object::Map(ref m) => {
            Ok(Map::new_object(m.borrow().m.clone()))
        },
        ref x => {
            Ok(x.clone())
        }
    }
}

fn rng_float(seed: u32) -> FnResult {
    let mut rng = Rand::new(seed);
    let f = Box::new(move |_: &mut Env, _: &Object, _: &[Object]| -> FnResult {
        Ok(Object::Float(rng.rand_float()))
    });
    return Ok(Function::mutable(f,0,0));
}

fn rng_range(env: &mut Env, r: &Range, seed: u32) -> FnResult {
    let a = match r.a {
        Object::Int(x)=>x,
        _ => return env.type_error1(
            "Type error in rng(a..b): a is not an integer.",
            "a",&r.a)
    };
    let b = match r.b {
        Object::Int(x)=>x,
        _ => return env.type_error1(
            "Type error in rng(a..b): b is not an integer.",
            "b",&r.b)
    };
    let mut rng = Rand::new(seed);
    let f = Box::new(move |_: &mut Env, _: &Object, _: &[Object]| -> FnResult {
        Ok(Object::Int(rng.rand_range(a,b)))
    });
    return Ok(Function::mutable(f,0,0));
}

fn rng_list(env: &mut Env, a: Rc<RefCell<List>>,
    seed: u32
) -> FnResult {
    let len = a.borrow_mut().v.len();
    let n = if len>0 {(len-1) as i32} else {
        return env.value_error("Value error in rng(a): size(a)==0.");
    };
    let mut rng = Rand::new(seed);
    let f = Box::new(move |_: &mut Env, _: &Object, _: &[Object]| -> FnResult {
        let index = rng.rand_range(0,n) as usize;
        Ok(a.borrow_mut().v[index].clone())
    });
    return Ok(Function::mutable(f,0,0));    
}

fn seed_arg(argm: &HashMap<Object,Object>) -> Option<u32> {
    if argm.len() != 1 {return None;}
    for (key, value) in argm {
        if let Object::String(key) = key {
            if key.data == ['s','e','e','d'] {
                if let Object::Int(seed) = *value {
                    return Some(seed as u32);
                }
            }
        }
    }
    None
}

fn rng(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    let mut len = argv.len();
    let seed = match argv.last() {
        Some(obj) => {
            if let Object::Int(seed) = *obj {
                Some(seed as u32)
            } else if let Object::Map(argm) = obj {
                seed_arg(&argm.borrow().m)
            } else {
                None
            }
        },
        None => None
    };
    let seed = match seed {
        Some(seed) => {len -= 1; seed},
        None => env.rte().seed_rng.borrow_mut().rand_u32()
    };
    match len {
        0 => rng_float(seed),
        1 => {
            if let Some(r) = downcast::<Range>(&argv[0]) {
                return rng_range(env,r,seed);
            }else if let Object::List(ref a) = argv[0] {
                return rng_list(env,a.clone(),seed);
            }
            return env.type_error1(
                "Type error in rng(r): r is not a range.",
                "r", &argv[0]
            );
        },
        n => env.argc_error(n,0,1,"rng")
    }
}

fn fgtab(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    if argv.len()==1 {
        match argv[0] {
            Object::Function(ref f) => {
                if let EnumFunction::Std(ref fp) = f.f {
                    Ok(Object::Map(fp.gtab.clone()))
                }else{
                    env.type_error("Type error in gtab(f): f is not a function from Moss source code.")
                }
            },
            _ => env.type_error1(
                "Type error in gtab(f): f is not a function.",
                "f",&argv[0])
        }
    }else{
        Ok(Object::Map(env.rte().pgtab.borrow().clone()))
    }
}

enum ParseError{Invalid,Overflow}

fn parse_int(a: &[char]) -> Result<i32,ParseError> {
    let n = a.len();
    let mut sgn: i32 = 1;
    let mut i = 0;
    while i<n && a[i]==' ' {i+=1;}
    if i<n && a[i]=='-' {i+=1; sgn = -1;}
    let mut y = 0;
    let base: u32 = if i+1<n && a[i]=='0' {
        match a[i+1] {'x' => 16, 'b' => 2, 'o' => 8, _ => 10}
    } else {10};
    if base != 10 {i+=2;}
    if i==n {return Err(ParseError::Invalid);}
    while i<n {
        let x = a[i];
        match x.to_digit(base) {
            Some(digit) => {
                match (base as i32).checked_mul(y) {
                    Some(value) => {y = value;},
                    None => return Err(ParseError::Overflow)
                }
                match (y).checked_add(digit as i32) {
                    Some(value) => {y = value;},
                    None => return Err(ParseError::Overflow)
                }
            },
            None => {
                if x != '_' {
                    for &x in &a[i..] {
                        if x != ' ' {return Err(ParseError::Invalid);}
                    }
                }
            }
        };
        i+=1;
    }
    return Ok(sgn*y);
}

fn string_to_int(env: &mut Env, a: &[char]) -> FnResult {
    match parse_int(a) {
        Ok(x) => return Ok(Object::Int(x)),
        Err(e) => match e {
            ParseError::Invalid => {},
            ParseError::Overflow => {
                match Long::object_from_string(a) {
                    Ok(value) => return Ok(value),
                    Err(()) => {}
                }
            }
        }
    }
    return env.value_error(
        "Value error in int(s): could not convert s into an integer.");
}

fn int(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"int")
    }
    match argv[0] {
        Object::Bool(x) => return Ok(Object::Int(x as i32)),
        Object::Int(n) => return Ok(Object::Int(n)),
        Object::Float(x) => return Ok(Object::Int(x.round() as i32)),
        Object::String(ref s) => return string_to_int(env,&s.data),
        _ => {}
    }
    if let Some(x) = downcast::<Long>(&argv[0]) {
        match Long::try_as_int(x) {
            Ok(x) => return Ok(Object::Int(x)),
            Err(()) => return Ok(argv[0].clone())
        }
    }
    env.type_error1(
        "Type error in int(x): cannot convert x to int.",
        "x", &argv[0])
}

fn to_float(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"float")
    }
    match argv[0] {
        Object::Int(n) => return Ok(Object::Float(float(n))),
        Object::Float(x) => return Ok(Object::Float(x)),
        Object::Complex(z) => return Ok(Object::Float(
            if z.im==0.0 {z.re} else {NAN}
        )),
        Object::String(ref s) => {
            return match f64::from_str(&s.to_string()) {
                Ok(value) => Ok(Object::Float(value)),
                Err(_) => env.value_error("Value error: parse error in float(s).")
            }
        },
        _ => {}
    }
    if let Some(x) = downcast::<Long>(&argv[0]) {
        return Ok(Object::Float(x.as_f64()));
    }
    env.type_error1(
        "Type error in float(x): cannot convert x to float.",
        "x", &argv[0])
}

fn input(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        0 => {
            let s = match crate::system::getline("") {
                Ok(s)=>s, Err(_) => return env.std_exception("Error in input(): could not obtain input.")
            };
            Ok(CharString::new_object_str(&s))
        },
        1|2 => {
            let prompt = match argv[0] {
                Object::String(ref s) => s.to_string(),
                _ => return env.type_error1(
                    "Type error in input(prompt): prompt is not a string.",
                    "prompt",&argv[0])
            };
            let s = if argv.len()==2 {
                if let Object::List(ref a) = argv[1] {
                    let mut h = History::new();
                    for x in &a.borrow().v {
                        h.append(&x.string(env)?);
                    }
                    match crate::system::getline_history(&prompt,&h) {
                        Ok(s)=>s, Err(_) => return env.std_exception("Error in input(): could not obtain input.")
                    }
                }else{
                    return env.type_error1(
                        "Type error in input(prompt,history): history is not a list.",
                        "history",&argv[1])
                }
            }else{
                match crate::system::getline(&prompt) {
                    Ok(s)=>s, Err(_) => return env.std_exception("Error in input(): could not obtain input.")
                }
            };
            Ok(CharString::new_object_str(&s))
        },
        n => {
            env.argc_error(n,0,2,"input")
        }
    }
}

fn fconst(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"const")
    }
    match argv[0] {
        Object::List(ref a) => {
            let mut a = a.borrow_mut();
            a.frozen = true;
        },
        Object::Map(ref m) => {
            let mut m = m.borrow_mut();
            m.frozen = true;
        },
        Object::Interface(ref x) => {
            if let Some(t) = x.as_any().downcast_ref::<Table>() {
                let mut m = t.map.borrow_mut();
                m.frozen = true;
            }
        },
        _ => {}
    }
    return Ok(argv[0].clone());
}

fn read(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"read")
    }
    match argv[0] {
        Object::String(ref id) => {
            let id = id.to_string();
            if !env.rte().read_access(&id) {
                return env.std_exception(&format!(
                    "Error in read(id): Could not open file id=='{}': permission denied.",id
                ));
            }
            
            return match crate::system::read_file(&id) {
                Ok(s) => Ok(CharString::new_object_str(&s)),
                Err(e) => env.std_exception(&e)
            }
        },
        ref x => env.type_error1(
            "Type error in read(id): id is not a string.",
            "id", x)
    }
}

fn _zip(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    let argc = argv.len();
    if argc==0 {
        return Ok(List::new_object(Vec::new()));
    }
    let mut v: Vec<Rc<RefCell<List>>> = Vec::with_capacity(argc);
    for i in 0..argc {
        match argv[i] {
            Object::List(ref a) => v.push(a.clone()),
            ref a => {
                let y = list(env,a)?;
                // todo: traceback
                v.push(match y {Object::List(a) => a, _ => unreachable!()});
            }
        }
    }
    let n = v[0].borrow().v.len();
    for i in 0..argc {
        if n != v[i].borrow().v.len() {
            return env.type_error("Type error in f[a1,...,an]: all lists must have the same size.");
        }
    }
    let mut vy: Vec<Object> = Vec::with_capacity(argc);
    for k in 0..n {
        let mut t: Vec<Object> = Vec::with_capacity(argc);
        for i in 0..argc {
            t.push(v[i].borrow().v[k].clone());
        }
        vy.push(List::new_object(t));
    }
    return Ok(List::new_object(vy));
}

fn zip(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    let argc = argv.len();
    let mut v: Vec<Object> = Vec::with_capacity(argc);
    for k in 0..argc {
        let i = iter(env,&argv[k])?;
        v.push(i);
    }
    let g = Box::new(move |env: &mut Env, _pself: &Object, _argv: &[Object]| -> FnResult {
        let mut t: Vec<Object> = Vec::with_capacity(argc);
        for i in &v {
            let y = env.call(i,&Object::Null,&[])?;
            if y.is_empty() {return Ok(y);}
            else {t.push(y);}
        }
        return Ok(List::new_object(t));
    });
    return Ok(new_iterator(g));
}

fn pow(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        3 => {}, n => return env.argc_error(n,3,3,"pow")
    }
    return pow_mod(env,&argv[0],&argv[1],&argv[2]);
}

fn min(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        2 => {}, n => return env.argc_error(n,2,2,"min")
    }
    let cond = op_lt(env,&argv[0],&argv[1])?;
    if let Object::Bool(cond) = cond {
        return Ok(argv[if cond {0} else {1}].clone());
    }else{
        return env.type_error("Type error in min(x,y): value of x<y is not a boolean.")
    }
}

fn max(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        2 => {}, n => return env.argc_error(n,2,2,"max")
    }
    let cond = op_lt(env,&argv[0],&argv[1])?;
    if let Object::Bool(cond) = cond {
        return Ok(argv[if cond {1} else {0}].clone());
    }else{
        return env.type_error("Type error in max(x,y): value of x<y is not a boolean.")
    }
}

fn ord(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"ord")
    }
    match argv[0] {
        Object::String(ref s) => {
            if s.data.len()==1 {
                return Ok(Object::Int(s.data[0] as u32 as i32));
            }else{
                env.value_error("Value error in ord(c): size(c)!=1.")
            }
        },
        ref c => env.type_error1("Type error in ord(c): c is not a string.","c",c)
    }
}

fn chr(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"chr")
    }
    match argv[0] {
        Object::Int(n) => {
            Ok(match char::from_u32(n as u32) {
                Some(c) => CharString::new_object_char(c),
                None => Object::Null
            })
        },
        ref n => env.type_error1(
            "Type error in chr(n): n is not an integer.","n",n)
    }
}

fn map(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"map")
    }
    let i = iter(env,&argv[0])?;
    let mut m: HashMap<Object,Object> = HashMap::new();
    loop{
        let y = env.call(&i,&Object::Null,&[])?;
        if y.is_empty() {
            break;
        }else if let Object::List(a) = y {
            let a = a.borrow_mut();
            if a.v.len() != 2 {
                return env.type_error("Type error in map(a): iter(a) is expected to return pairs.");
            }
            m.insert(a.v[0].clone(),a.v[1].clone());
        }else{
            return env.type_error("Type error in map(a): iter(a) is expected to return lists.");
        }
    }
    return Ok(Map::new_object(m));
}

fn extend(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    if argv.len()<2 {
        return env.argc_error(argv.len(),2,VARIADIC,"extend");
    }
    let mut borrow = if let Some(t) = downcast::<Table>(&argv[0]) {
        t.map.borrow_mut()
    }else if let Some(t) = downcast::<Class>(&argv[0]) {
        t.map.borrow_mut()
    }else{
        return env.type_error("Type error in extend(x,y): x is not a table.");
    };
    let m = &mut borrow;
    for p in &argv[1..] {
        if let Object::Map(ref pm) = *p {
            let pm = &pm.borrow();
            map_extend(m,pm);
        }else if let Some(pt) = downcast::<Table>(p) {
            let pm = &pt.map.borrow();
            map_extend(m,pm);
        }else if let Some(pc) = downcast::<Class>(p) {
            let pm = &pc.map.borrow();
            map_extend(m,pm);
        }else{
            return env.type_error("Type error in extend(x,y): y is not a table.");
        }
    }
    return Ok(Object::Null);
}

fn long(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"long")
    }
    match Long::to_long(&argv[0]) {
        Ok(y) => Ok(y),
        Err(()) => {
            env.type_error1(
                "Type error in long(x): cannot convert x to long.",
                "x",&argv[0]
            )
        }
    }
}

fn abort(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        0 => return env.std_exception("Aborted."),
        1 => return env.std_exception(&format!("Aborted: {}.",argv[0])),
        n => return env.argc_error(n,0,1,"abort")
    }
}

fn getattr(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        2 => {}, n => return env.argc_error(n,2,2,"getattr")
    }
    Ok(if let Some(t) = downcast::<Table>(&argv[0]) {
        match table_get(&t,&argv[1]) {
            Some(x) => x,
            _ => Object::Null
        }
    }else{
        Object::Null
    })
}

fn hex(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"hex")
    }
    if let Object::Int(n) = argv[0] {
        let s: &str = &format!("0x{:x}",n);
        Ok(Object::from(s))
    }else if let Some(n) = downcast::<Long>(&argv[0]) {
        let s: &str = &format!("0x{}",Long::to_hex(n));
        Ok(Object::from(s))
    }else if let Some(data) = downcast::<Bytes>(&argv[0]) {
        Ok(base16(&data.data.borrow()))
    }else{
        env.type_error1(
           "Type error in hex(x): cannot convert x into hexadecimal representation.",
           "x",&argv[0])
    }
}

fn bin(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"bin")
    }
    if let Object::Int(n) = argv[0] {
        let s: &str = &format!("0b{:b}",n);
        Ok(Object::from(s))
    }else{
        env.type_error1(
           "Type error in bin(x): cannot convert x into binary representation.",
           "x",&argv[0])
    }
}

fn oct(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"oct")
    }
    if let Object::Int(n) = argv[0] {
        let s: &str = &format!("0o{:o}",n);
        Ok(Object::from(s))
    }else{
        env.type_error1(
           "Type error in oct(x): cannot convert x into octal representation.",
           "x",&argv[0])
    }
}

pub fn init_rte(rte: &RTE){
    let mut gtab = rte.gtab.borrow_mut();
    gtab.insert_fn_plain("print",print,0,VARIADIC);
    gtab.insert_fn_plain("put",put,0,VARIADIC);
    gtab.insert_fn_plain("str",fstr,1,1);
    gtab.insert_fn_plain("int",int,1,1);
    gtab.insert_fn_plain("float",to_float,1,1);
    gtab.insert_fn_plain("repr",repr,1,1);
    gtab.insert_fn_plain("hex",hex,1,1);
    gtab.insert_fn_plain("bin",bin,1,1);
    gtab.insert_fn_plain("oct",oct,1,1);
    gtab.insert_fn_plain("input",input,0,2);
    gtab.insert_fn_plain("sgn",sgn,1,1);
    gtab.insert_fn_plain("abs",abs,1,1);
    gtab.insert_fn_plain("eval",eval,1,1);
    gtab.insert_fn_plain("len",len,1,1);
    gtab.insert_fn_plain("load",fload,1,1);
    gtab.insert_fn_plain("iter",fiter,1,1);
    gtab.insert_fn_plain("cycle",cycle,1,1);
    gtab.insert_fn_plain("record",record,1,1);
    gtab.insert_fn_plain("object",fobject,0,2);
    gtab.insert_fn_plain("type",ftype,1,1);
    gtab.insert_fn_plain("list",flist,1,1);
    gtab.insert_fn_plain("set",set,1,1);
    gtab.insert_fn_plain("copy",copy,1,1);
    gtab.insert_fn_plain("rng",rng,1,1);
    gtab.insert_fn_plain("gtab",fgtab,0,0);
    gtab.insert_fn_plain("const",fconst,1,1);
    gtab.insert_fn_plain("read",read,1,1);
    gtab.insert_fn_plain("zip",zip,0,VARIADIC);
    gtab.insert_fn_plain("pow",pow,3,3);
    gtab.insert_fn_plain("min",min,2,2);
    gtab.insert_fn_plain("max",max,2,2);
    gtab.insert_fn_plain("ord",ord,1,1);
    gtab.insert_fn_plain("chr",chr,1,1);
    gtab.insert_fn_plain("map",map,1,1);
    gtab.insert_fn_plain("extend",extend,2,VARIADIC);
    gtab.insert_fn_plain("long",long,1,1);
    gtab.insert_fn_plain("abort",abort,0,1);
    gtab.insert_fn_plain("getattr",getattr,2,2);
    gtab.insert_fn_plain("class",class_new,1,1);
    gtab.insert("empty", Object::empty());

    let type_bool = rte.type_bool.clone();
    gtab.insert("Bool", Object::Interface(type_bool));
    
    let type_int = rte.type_int.clone();
    gtab.insert("Int", Object::Interface(type_int));
    
    let type_float = rte.type_float.clone();
    gtab.insert("Float", Object::Interface(type_float));
    
    let type_complex = rte.type_complex.clone();
    gtab.insert("Complex", Object::Interface(type_complex));

    let type_string = rte.type_string.clone();
    crate::string::init(&type_string);
    gtab.insert("String", Object::Interface(type_string));

    let type_list = rte.type_list.clone();
    crate::list::init(&type_list);
    gtab.insert("List", Object::Interface(type_list));

    let type_map = rte.type_map.clone();
    crate::map::init(&type_map);
    gtab.insert("Map", Object::Interface(type_map));

    let type_function = rte.type_function.clone();
    crate::function::init(&type_function);
    gtab.insert("Function", Object::Interface(type_function));
    
    let type_range = rte.type_range.clone();
    gtab.insert("Range", Object::Interface(type_range));

    let type_iterable = rte.type_iterable.clone();
    crate::iterable::init(&type_iterable);
    gtab.insert("Iterable", Object::Interface(type_iterable));

    let type_long = rte.type_long.clone();
    gtab.insert("Long", Object::Interface(type_long));

    let type_exception = rte.type_exception.clone();
    gtab.insert("Exception", Object::Interface(type_exception));

    let type_type_error = rte.type_type_error.clone();
    gtab.insert("TypeError", Object::Interface(type_type_error));

    let type_value_error = rte.type_value_error.clone();
    gtab.insert("ValueError", Object::Interface(type_value_error));

    let type_index_error = rte.type_index_error.clone();
    gtab.insert("IndexError", Object::Interface(type_index_error));

    let type_type = rte.type_type.clone();
    gtab.insert("Type", Object::Interface(type_type));

    let type_bytes = Class::new("Bytes",
        &Object::Interface(rte.type_iterable.clone())
    );
    {
        let mut m = type_bytes.map.borrow_mut();
        m.insert_fn_plain("list",crate::data::bytes_list,0,0);
        m.insert_fn_plain("decode",crate::data::bytes_decode,0,1);
        m.insert_fn_plain("len",crate::data::bytes_len,0,0);
        m.insert_fn_plain("hex",crate::data::bytes_hex,0,0);
    }
    interface_types_set(rte,interface_index::BYTES,type_bytes);
}

