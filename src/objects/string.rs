
use object::{
    Object, Table, FnResult, U32String
};
use vm::Env;

fn isdigit(c: char) -> bool {
    ('0' as u32)<=(c as u32) && (c as u32)<=('9' as u32)
}

fn isalpha(c: char) -> bool {
    let c = c as u32;
    ('A' as u32)<=c && c<=('Z' as u32) ||
    ('a' as u32)<=c && c<=('z' as u32)
}

fn isspace(c: char) -> bool {
    c==' ' || c=='\t' || c=='\n'
}

fn islower(c: char) -> bool {
    ('a' as u32)<=(c as u32) && (c as u32)<=('z' as u32)
}

fn isupper(c: char) -> bool {
    ('A' as u32)<=(c as u32) && (c as u32)<=('Z' as u32)
}

#[inline(never)]
fn type_error0_string(env: &mut Env, id: &str, s: &Object) -> FnResult {
     env.type_error1(&format!(
         "Type error in s.{}(): s is not a string.", id
     ),"s",s)
}

fn string_isdigit(env: &mut Env, pself: &Object, argv: &[Object])
-> FnResult
{
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"isdigit")
    }
    match *pself {
        Object::String(ref s) => {
            for c in &s.v {
                if !isdigit(*c) {return Ok(Object::Bool(false));}
            }
            return Ok(Object::Bool(true));
        },
        ref s => type_error0_string(env,"isdigit",s)
    }
}

fn string_isalpha(env: &mut Env, pself: &Object, argv: &[Object])
-> FnResult
{
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"isalpha")
    }
    match *pself {
        Object::String(ref s) => {
            for c in &s.v {
                if !isalpha(*c) {return Ok(Object::Bool(false));}
            }
            return Ok(Object::Bool(true));
        },
        ref s => type_error0_string(env,"isalpha",s)
    }
}

fn string_isalnum(env: &mut Env, pself: &Object, argv: &[Object])
-> FnResult
{
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"isalnum")
    }
    match *pself {
        Object::String(ref s) => {
            for c in &s.v {
                if !(isdigit(*c) || isalpha(*c)) {
                    return Ok(Object::Bool(false));
                }
            }
            return Ok(Object::Bool(true));
        },
        ref s => type_error0_string(env,"isalnum",s)
    }
}

fn string_isspace(env: &mut Env, pself: &Object, argv: &[Object])
-> FnResult
{
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"isspace")
    }
    match *pself {
        Object::String(ref s) => {
            for c in &s.v {
                if !isspace(*c) {return Ok(Object::Bool(false));}
            }
            return Ok(Object::Bool(true));
        },
        ref s => type_error0_string(env,"isspace",s)
    }
}

fn string_islower(env: &mut Env, pself: &Object, argv: &[Object])
-> FnResult
{
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"islower")
    }
    match *pself {
        Object::String(ref s) => {
            for c in &s.v {
                if !islower(*c) {return Ok(Object::Bool(false));}
            }
            return Ok(Object::Bool(true));
        },
        ref s => type_error0_string(env,"islower",s)
    }
}

fn string_isupper(env: &mut Env, pself: &Object, argv: &[Object])
-> FnResult
{
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"isupper")
    }
    match *pself {
        Object::String(ref s) => {
            for c in &s.v {
                if !isupper(*c) {return Ok(Object::Bool(false));}
            }
            return Ok(Object::Bool(true));
        },
        ref s => type_error0_string(env,"isupper",s)
    }
}

fn lower(env: &mut Env, pself: &Object, argv: &[Object])
-> FnResult
{
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"lower")
    }
    match *pself {
        Object::String(ref s) => {
            let mut v: Vec<char> = Vec::new();
            for c in &s.v {
                v.append(&mut c.to_lowercase().collect());
            }
            return Ok(U32String::new_object(v));
        },
        ref s => type_error0_string(env,"lower",s)
    }
}

fn upper(env: &mut Env, pself: &Object, argv: &[Object])
-> FnResult
{
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"upper")
    }
    match *pself {
        Object::String(ref s) => {
            let mut v: Vec<char> = Vec::new();
            for c in &s.v {
                v.append(&mut c.to_uppercase().collect());
            }
            return Ok(U32String::new_object(v));
        },
        ref s => type_error0_string(env,"upper",s)
    }
}

fn ljust(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    let c = match argv.len() {
        1 => {' '},
        2 => {
            match argv[1] {
                Object::String(ref s) => {
                    if s.v.len()==1 {s.v[0]} else {
                        return env.value_error(
                            "Value error in s.ljust(n,c): size(c)!=1."
                        );
                    }
                },
                _ => {
                    return env.type_error1(
                        "Type error in s.ljust(n,c): c is not a string.",
                        "c", &argv[1]
                    );
                }
            }
        },
        n => return env.argc_error(n,2,2,"ljust")
    };
    let s = match *pself {
        Object::String(ref s) => &s.v,
        _ => {return env.type_error1(
            "Type error in s.ljust(n): s is not a string.",
            "s",pself
        );}
    };
    let n = match argv[0] {
        Object::Int(x) => {
            if x<0 {0} else{x as usize}
        },
        _ => {return env.type_error1(
            "Type error in s.ljust(n): n is not an integer.",
            "s",pself
        );}
    };
    let mut v: Vec<char> = s.clone();
    for _ in s.len()..n {
        v.push(c);
    }
    return Ok(U32String::new_object(v));
}

fn rjust(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    let c = match argv.len() {
        1 => {' '},
        2 => {
            match argv[1] {
                Object::String(ref s) => {
                    if s.v.len()==1 {s.v[0]} else {
                        return env.value_error(
                            "Value error in s.rjust(n,c): size(c)!=1."
                        );
                    }
                },
                _ => {
                    return env.type_error1(
                        "Type error in s.rjust(n,c): c is not a string.",
                        "c",&argv[1]
                    );
                }
            }
        },
        n => return env.argc_error(n,2,2,"ljust")
    };
    let s = match *pself {
        Object::String(ref s) => &s.v,
        _ => {return env.type_error1(
            "Type error in s.rjust(n): s is not a string.",
            "s", pself
        );}
    };
    let n = match argv[0] {
        Object::Int(x) => {
            if x<0 {0} else{x as usize}
        },
        _ => {return env.type_error1(
            "Type error in s.rjust(n): n is not an integer.",
            "s", pself
        );}
    };
    let mut v: Vec<char> = Vec::new();
    for _ in s.len()..n {
        v.push(c);
    }
    for x in s {
        v.push(*x);
    }
    return Ok(U32String::new_object(v));
}

pub fn duplicate(s: &[char], n: i32) -> Object {
    if n<0 {
        return U32String::new_object_str("");
    }else{
        let mut v: Vec<char> = Vec::with_capacity(n as usize*s.len());
        for _ in 0..n {
            v.extend_from_slice(&s);
        }
        return U32String::new_object(v);
    }
}

pub fn init(t: &Table){
    let mut m = t.map.borrow_mut();
    m.insert_fn_plain("isdigit",string_isdigit,0,0);
    m.insert_fn_plain("isalpha",string_isalpha,0,0);
    m.insert_fn_plain("isalnum",string_isalnum,0,0);
    m.insert_fn_plain("isspace",string_isspace,0,0);
    m.insert_fn_plain("islower",string_islower,0,0);
    m.insert_fn_plain("isupper",string_isupper,0,0);
    m.insert_fn_plain("lower",lower,0,0);
    m.insert_fn_plain("upper",upper,0,0);
    m.insert_fn_plain("ljust",ljust,1,2);
    m.insert_fn_plain("rjust",rjust,1,2);
}
