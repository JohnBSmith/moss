
use crate::object::{Object, FnResult, CharString, Exception};
use crate::vm::Env;

fn get(env: &Env, a: &Object, i: usize) -> FnResult {
    match *a {
        Object::List(ref a) => {
            let v = &a.borrow_mut().v;
            match v.get(i) {
                Some(value) => Ok(value.clone()),
                None => env.index_error(
                    "Index error in a[i]: out of bounds.")
            }
        },
        Object::Map(ref m) => {
            let d = &m.borrow_mut().m;
            match d.get(&Object::Int(i as i32)) {
                Some(value) => Ok(value.clone()),
                None => env.index_error(
                    "Index error in m[key]: key not found.")
            }
        },
        _ => env.type_error("Type error in a[i]: a is not a list.")
    }
}

fn get_key(env: &Env, m: &Object, key: &Object) -> FnResult {
    match *m {
        Object::Map(ref m) => {
            let d = &m.borrow_mut().m;
            match d.get(key) {
                Some(value) => Ok(value.clone()),
                None => env.index_error(&format!(
                    "Index error in m[{0}]: {0} not found.", key.to_repr()
                ))
            }
        },
        _ => env.type_error("Type error in m[key]: m is not a map.")
    }
}

enum Space {
    None, Left(usize), Center(usize), Right(usize)
}

struct Float {
    fmt: char,
    precision: Option<usize>
}

enum FmtType {
    None, Int(char), Float(Float)
}

struct Fmt {
    space: Space, fmt_type: FmtType, sign: bool, fill: char
}

impl Fmt {
    fn new() -> Self {
        Fmt{
            space: Space::None,
            fmt_type: FmtType::None,
            sign: false,
            fill: ' '
        }
    }
}

fn number(v: &[char], mut i: usize, value: &mut usize) -> usize {
    let n = v.len();
    while i<n && v[i]==' ' {i+=1;}
    let mut x: usize = 0;
    if i<n && v[i].is_digit(10) {
        x = v[i] as usize - '0' as usize;
        i+=1;
    }else{
        *value = x;
        return i;
    }
    while i<n && v[i].is_digit(10) {
        x = 10*x + v[i] as usize - '0' as usize;
        i+=1;
    }
    *value = x;
    return i;
}

fn obtain_fmt(fmt: &mut Fmt, v: &[char], mut i: usize)
-> Result<usize,String>
{
    let n = v.len();
    while i<n && v[i]==' ' {i+=1;}
    if i>=n {return Ok(i);}
    let mut value: usize = 0;
    if v[i]=='l' {
        i+=1;
        i = number(v,i,&mut value);
        fmt.space = Space::Left(value);
    }else if v[i]=='r' {
        i+=1;
        i = number(v,i,&mut value);
        fmt.space = Space::Right(value);
    }else if v[i]=='c' {
        i+=1;
        i = number(v,i,&mut value);
        fmt.space = Space::Center(value);
    }
    if i<n && (v[i]=='+' || v[i]=='-') {
        fmt.sign = true;
        i+=1;
    }
    if i+2<n && v[i]=='(' && v[i+2]==')' {
        fmt.fill = v[i+1];
        i+=3;
    }
    if i<n && (v[i]=='f' || v[i]=='e') {
        let c = v[i];
        i+=1;
        let precision = if i<n && v[i].is_digit(10) {
            i = number(v,i,&mut value);
            Some(value)
        }else{
            None
        };
        fmt.fmt_type = FmtType::Float(Float{fmt: c, precision});
    }else if i<n && (v[i]=='x' || v[i]=='o' || v[i]=='b') {
        fmt.fmt_type = FmtType::Int(v[i]);
        i+=1;
    }
    if i<n {
        if v[i]=='}' {
            return Ok(i);
        }else{
            return Err(format!(
                "Value error in s%a: in s: unexpected character: '{}'.", v[i]
            ));
        }
    }else{
        return Err(String::from(
            "Value error in s%a: in s: expected '}'."
        ));
    }
}

fn apply_fmt(env: &mut Env, buffer: &mut String,
    fmt: &Fmt, x: &Object
) -> Result<(),Box<Exception>>
{
    let s = match fmt.fmt_type {
        FmtType::None => x.string(env)?,
        FmtType::Int(mode) => {
            if let Object::Int(n) = *x {
                if mode == 'x' {format!("{:x}",n)}
                else if mode == 'b' {format!("{:b}",n)}
                else if mode == 'o' {format!("{:o}",n)}
                else {unreachable!()}
            }else{
                panic!();
            }
        },
        FmtType::Float(ref float) => {
            let x = match *x {
                Object::Int(n) => n as f64,
                Object::Float(x) => x,
                _ => {
                    return match env.type_error("Type error in format: expected a float.") {
                        Ok(_) => unreachable!(),
                        Err(e) => Err(Box::new(*e))
                    }
                }
            };
            if float.fmt == 'f' {
                if let Some(precision) = float.precision {
                    if fmt.sign {
                        format!("{:+.*}",precision,x)
                    }else{
                        format!("{:.*}",precision,x)
                    }
                }else{
                    if fmt.sign {
                        format!("{:+}",x)
                    }else{
                        format!("{:}",x)
                    }
                }
            }else if float.fmt == 'e' {
                if let Some(precision) = float.precision {
                    if fmt.sign {
                        format!("{:+.*e}",precision,x)
                    }else{
                        format!("{:.*e}",precision,x)
                    }
                }else{
                    if fmt.sign {
                        format!("{:+e}",x)
                    }else{
                        format!("{:e}",x)
                    }
                }
            }else{
                unreachable!()
            }
        }
    };
    match fmt.space {
        Space::Left(value) => {
            buffer.push_str(&s);
            for _ in s.len()..value {
                buffer.push(fmt.fill);
            }
        },
        Space::Right(value) => {
            for _ in s.len()..value {
                buffer.push(fmt.fill);
            }    
            buffer.push_str(&s);
        },
        _ => {
            buffer.push_str(&s);
        }
    }
    return Ok(());
}

pub fn u32string_format(env: &mut Env, s: &CharString, a: &Object)
-> FnResult
{
    let mut buffer = "".to_string();
    let mut index: usize = 0;
    let mut i: usize = 0;
    let v = &s.data;
    let n = v.len();
    while i<n {
        let c = v[i];
        if c=='{' {
            if v[i+1]=='{' {
                buffer.push('{');
                i+=2;
            }else {
                let mut fmt = Fmt::new();
                i+=1;
                while i<n && v[i]==' ' {i+=1;}
                let x: Object;
                if i<n && v[i].is_alphabetic() {
                    let j = i;
                    while i<n && (
                        v[i].is_alphabetic()
                        || v[i].is_digit(10)
                        || v[i]=='_'
                    ) {
                        i+=1;
                    }
                    let key = CharString::new_object(v[j..i].iter().cloned().collect());
                    x = get_key(env,&a,&key)?;
                }else if i<n && v[i].is_digit(10) {
                    let mut j: usize = v[i] as usize-'0' as usize;
                    i+=1;
                    while i<n && v[i].is_digit(10) {
                        j = 10*j + v[i] as usize-'0' as usize;
                        i+=1;
                    }
                    x = get(env,&a,j)?;
                }else{
                    x = get(env,&a,index)?;
                    index+=1;    
                }
                while i<n && v[i]==' ' {i+=1;}
                if i<n && v[i]==':' {i+=1;}
                i = match obtain_fmt(&mut fmt,v,i) {
                    Ok(index) => index,
                    Err(s) => {
                        return env.value_error(&s)
                    }
                };
                apply_fmt(env,&mut buffer,&fmt,&x)?;
                while i<n && v[i]==' ' {i+=1;}
                if i<n && v[i]=='}' {i+=1;}
            }
        }else if c=='}' && i+1<n && v[i+1]=='}' {
            buffer.push('}');
            i+=2;
        }else{
            buffer.push(c);
            i+=1;
        }
    }
    return Ok(CharString::new_object_str(&buffer));
}
