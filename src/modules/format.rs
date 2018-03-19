
#![allow(unused_imports)]

use object::{Object, FnResult, U32String, Exception};
use vm::Env;

fn get(env: &Env, a: &Object, i: usize) -> FnResult {
  match *a {
    Object::List(ref a) => {
      let v = &a.borrow_mut().v;
      match v.get(i) {
        Some(value) => Ok(value.clone()),
        None => env.index_error("Index error in a[i]: out of bounds.")
      }
    },
    Object::Map(ref m) => {
      let d = &m.borrow_mut().m;
      match d.get(&Object::Int(i as i32)) {
        Some(value) => Ok(value.clone()),
        None => env.index_error("Index error in m[key]: key not found.")
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
        None => env.index_error("Index error in m[key]: key not found.")
      }
    },
    _ => env.type_error("Type error in m[key]: m is not a map.")
  }
}

enum Space {
  None, Left(usize), Center(usize), Right(usize)
}

struct Fmt {
  space: Space
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

fn obtain_fmt(fmt: &mut Fmt, v: &[char], mut i: usize) -> Result<usize,Box<Exception>> {
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
  return Ok(i);
}

fn apply_fmt(env: &mut Env, buffer: &mut String,
  fmt: &Fmt, x: &Object
) -> Result<(),Box<Exception>>
{
  let s = try!(x.string(env));
  match fmt.space {
    Space::Left(value) => {
      buffer.push_str(&s);
      for _ in s.len()..value {
        buffer.push(' ');
      }
    },
    Space::Right(value) => {
      for _ in s.len()..value {
        buffer.push(' ');
      }    
      buffer.push_str(&s);
    },
    _ => {
      buffer.push_str(&s);
    }
  }
  return Ok(());
}

pub fn u32string_format(env: &mut Env, s: &U32String, a: &Object) -> FnResult {
  let mut buffer = "".to_string();
  let mut index: usize = 0;
  let mut i: usize = 0;
  let v = &s.v;
  let n = v.len();
  while i<n {
    let c = v[i];
    if c=='{' {
      if v[i+1]=='{' {
        buffer.push('{');
        i+=2;
      }else {
        let mut fmt = Fmt{space: Space::None};
        i+=1;
        while i<n && v[i]==' ' {i+=1;}
        let x: Object;
        if i<n && v[i].is_alphabetic() {
          let j = i;
          while i<n && (v[i].is_alphabetic() || v[i].is_digit(10) || v[i]=='_') {i+=1;}
          let key = U32String::new_object(v[j..i].iter().cloned().collect());
          x = try!(get_key(env,&a,&key));
        }else if i<n && v[i].is_digit(10) {
          let mut j: usize = v[i] as usize-'0' as usize;
          i+=1;
          while i<n && v[i].is_digit(10) {
            j = 10*j + v[i] as usize-'0' as usize;
            i+=1;
          }
          x = try!(get(env,&a,j));
        }else{
          x = try!(get(env,&a,index));
          index+=1;    
        }
        while i<n && v[i]==' ' {i+=1;}
        if i<n && v[i]==':' {i+=1;}
        i = try!(obtain_fmt(&mut fmt,v,i));
        try!(apply_fmt(env,&mut buffer,&fmt,&x));
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
  return Ok(U32String::new_object_str(&buffer));
}
