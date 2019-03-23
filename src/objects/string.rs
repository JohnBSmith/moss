
use crate::object::{
    Object, Table, FnResult, CharString
};
use crate::vm::Env;
use crate::data::Bytes;

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
    let base = match argv.len() {
        0 => 10 as u32,
        1 => match argv[0] {
          Object::Int(x) => if x<0 {0 as u32} else {x as u32},
          ref x => return env.type_error1(
            "Type error in s.isdigit(base): base is not an integer.",
            "base", x
          )
        },
        n => return env.argc_error(n,0,1,"isdigit")
    };
    match *pself {
        Object::String(ref s) => {
            for c in &s.data {
                if !(*c).is_digit(base) {return Ok(Object::Bool(false));}
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
            for c in &s.data {
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
            for c in &s.data {
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
            for c in &s.data {
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
            for c in &s.data {
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
            for c in &s.data {
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
            for c in &s.data {
                v.append(&mut c.to_lowercase().collect());
            }
            return Ok(CharString::new_object(v));
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
            for c in &s.data {
                v.append(&mut c.to_uppercase().collect());
            }
            return Ok(CharString::new_object(v));
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
                    if s.data.len()==1 {s.data[0]} else {
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
        Object::String(ref s) => &s.data,
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
    return Ok(CharString::new_object(v));
}

fn rjust(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    let c = match argv.len() {
        1 => {' '},
        2 => {
            match argv[1] {
                Object::String(ref s) => {
                    if s.data.len()==1 {s.data[0]} else {
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
        Object::String(ref s) => &s.data,
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
    return Ok(CharString::new_object(v));
}

pub fn duplicate(s: &[char], n: i32) -> Object {
    if n<0 {
        return CharString::new_object_str("");
    }else{
        let mut v: Vec<char> = Vec::with_capacity(n as usize*s.len());
        for _ in 0..n {
            v.extend_from_slice(&s);
        }
        return CharString::new_object(v);
    }
}

fn contains(chars: &[char], c: char) -> bool {
    for x in chars {
        if *x==c {return true;}
    }
    return false;
}

fn ltrim(s: &[char], chars: &[char]) -> Vec<char> {
    let mut i = 0;
    let n = s.len();
    while i<n && contains(chars,s[i]) {
        i+=1;
    }
    return Vec::from(&s[i..]);
}

fn rtrim(s: &[char], chars: &[char]) -> Vec<char> {
    let mut j = s.len();
    while j>0 && contains(chars,s[j-1]) {
        j-=1;
    }
    return Vec::from(&s[..j]);
}

fn trim(s: &[char], chars: &[char]) -> Vec<char> {
    let mut i = 0;
    let mut j = s.len();
    while i<j && contains(chars,s[i]) {
        i+=1;
    }
    while j>i && contains(chars,s[j-1]) {
        j-=1;
    }
    return Vec::from(&s[i..j]);
}

fn string_ltrim(env: &mut Env, pself: &Object, argv: &[Object])
-> FnResult
{
    let s = match *pself {
        Object::String(ref s) => &s.data,
        _ => return env.type_error("Type error in s.ltrim(): s is not a string.")
    };
    match argv.len() {
        0 => {
            let v = ltrim(s,&[' ','\n','\t']);
            Ok(CharString::new_object(v))
        },
        1 => match argv[0] {
            Object::String(ref chars) => {
                let v = ltrim(s,&chars.data);
                Ok(CharString::new_object(v))
            },
            _ => env.type_error("Type error in s.ltrim(chars): chars is not a string.")
        },
        n => env.argc_error(n,0,1,"ltrim")
    }
}

fn string_rtrim(env: &mut Env, pself: &Object, argv: &[Object])
-> FnResult
{
    let s = match *pself {
        Object::String(ref s) => &s.data,
        _ => return env.type_error("Type error in s.rtrim(): s is not a string.")
    };
    match argv.len() {
        0 => {
            let v = rtrim(s,&[' ','\n','\t']);
            Ok(CharString::new_object(v))
        },
        1 => match argv[0] {
            Object::String(ref chars) => {
                let v = rtrim(s,&chars.data);
                Ok(CharString::new_object(v))
            },
            _ => env.type_error("Type error in s.rtrim(chars): chars is not a string.")
        },
        n => env.argc_error(n,0,1,"rtrim")
    }
}

fn string_trim(env: &mut Env, pself: &Object, argv: &[Object])
-> FnResult
{
    let s = match *pself {
        Object::String(ref s) => &s.data,
        _ => return env.type_error("Type error in s.trim(): s is not a string.")
    };
    match argv.len() {
        0 => {
            let v = trim(s,&[' ','\n','\t']);
            Ok(CharString::new_object(v))
        },
        1 => match argv[0] {
            Object::String(ref chars) => {
                let v = trim(s,&chars.data);
                Ok(CharString::new_object(v))
            },
            _ => env.type_error("Type error in s.trim(chars): chars is not a string.")
        },
        n => env.argc_error(n,0,1,"trim")
    }
}

fn string_encode(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    let spec: String = match argv.len() {
        0 => String::from("utf-8"),
        1 => match argv[0] {
            Object::String(ref spec) => spec.to_string(),
            ref spec => return env.type_error1(
                "Type error in s.encode(spec): spec is not a string.",
                "spec",spec)
        },
        n => return env.argc_error(n,1,1,"encode")
    };
    match *pself {
        Object::String(ref s) => {
            if spec=="utf-8" {
                let data = s.to_string().into_bytes();
                Ok(Bytes::object_from_vec(data))
            }else{
                env.value_error(&format!(
                    "Value error in s.encode(spec): spec=='{}' is unknown.",spec.to_string()))
            }
        },
        ref s => env.type_error1(
            "Type error in s.encode(spec): s is not a string.","s",s)
    }
}

pub fn init(t: &Table){
    let mut m = t.map.borrow_mut();
    m.insert_fn_plain("isdigit",string_isdigit,0,1);
    m.insert_fn_plain("isalpha",string_isalpha,0,0);
    m.insert_fn_plain("isalnum",string_isalnum,0,0);
    m.insert_fn_plain("isspace",string_isspace,0,0);
    m.insert_fn_plain("islower",string_islower,0,0);
    m.insert_fn_plain("isupper",string_isupper,0,0);
    m.insert_fn_plain("lower",lower,0,0);
    m.insert_fn_plain("upper",upper,0,0);
    m.insert_fn_plain("ljust",ljust,1,2);
    m.insert_fn_plain("rjust",rjust,1,2);
    m.insert_fn_plain("ltrim",string_ltrim,0,1);
    m.insert_fn_plain("rtrim",string_rtrim,0,1);
    m.insert_fn_plain("trim",string_trim,0,1);
    m.insert_fn_plain("encode",string_encode,0,1);
}
