
#![allow(unused_variables)]
#![allow(dead_code)]

use std::rc::Rc;
use std::any::Any;
use std::ascii::AsciiExt;

use object::{
  Object, Table, Function, FnResult, Interface,
  new_module
};
use vm::Env;

#[derive(Debug,Clone,Copy)]
enum Class{
  Alpha, Digit, HexDigit, Lower, Upper,
  UnicodeAlpha
}

#[derive(Debug,Clone,Copy)]
struct CharClass {
  value: Class,
  neg: bool
}

#[derive(Debug)]
struct Range {
  a: char, b: char
}

#[derive(Debug)]
enum RegexSymbol {
  Dot, Char(char), Class(CharClass),
  Range(Box<Range>), Complement(Box<RegexSymbol>),
  Regex(Box<[RegexSymbol]>),
  Qm(Box<RegexSymbol>),
  Plus(Box<RegexSymbol>),
  Star(Box<RegexSymbol>),
  Or(Box<[RegexSymbol]>)
}

struct Token{
  line: usize,
  col: usize,
  index: usize,
  c: char
}

struct TokenIterator{
  v: Vec<Token>,
  index: usize
}

impl TokenIterator{
  fn get(&self) -> Option<char> {
    if self.index < self.v.len() {
      return Some(self.v[self.index].c);
    }else{
      return None;
    }
  }
  fn next_is(&self, k: usize, c: char) -> bool {
    if self.index+k < self.v.len() {
      return self.v[self.index+k].c==c;
    }else{
      return false;
    }
  }
  fn exists_at(&self, k: usize) -> bool {
    self.index+k < self.v.len()
  }
  fn at(&self, k: usize) -> char {
    self.v[self.index+k].c
  }
}

struct CharIterator<'a>{
  v: &'a Vec<char>,
  index: usize
}

impl<'a> CharIterator<'a> {
  fn get(&self) -> Option<char> {
    if self.index < self.v.len() {
      return Some(self.v[self.index]);
    }else{
      return None;
    }
  }
}

fn scan(s: &Vec<char>) -> Vec<Token> {
  let mut i: usize = 0;
  let mut line: usize = 0;
  let mut col: usize = 0;
  let n = s.len();
  let mut v: Vec<Token> = Vec::new();
  while i<n {
    let c = s[i];
    if c==' ' {
      col+=1;
      i+=1;
    }else if c=='\n' {
      line+=1;
      col=0;
      i+=1;
    }else{
      v.push(Token{
        c: c, index: i, line: line, col: col,
      });
      col+=1;
      i+=1;
    }
  }
  return v;
}

fn syntax_error(s: &str) -> Result<Option<RegexSymbol>,String> {
  return Err(format!("Syntax error in regex: {}",s));
}

fn new_char_class(neg: bool, value: Class) -> RegexSymbol {
  RegexSymbol::Class(CharClass{neg, value})
}

fn escape_seq(i: &mut TokenIterator) -> Result<Option<RegexSymbol>,String> {
  let neg = if i.next_is(0,'~') {
    i.index+=1; true
  }else{
    false
  };
  if let Some(c) = i.get() {
    let x = match c {
      's' => RegexSymbol::Char(' '),
      't' => RegexSymbol::Char('\t'),
      'n' => RegexSymbol::Char('\n'),
      'r' => RegexSymbol::Char('\r'),
      'L' => RegexSymbol::Char('{'),
      'R' => RegexSymbol::Char('}'),
      'd' => new_char_class(neg,Class::Digit),
      'x' => new_char_class(neg,Class::HexDigit),
      'l' => new_char_class(neg,Class::Lower),
      'u' => new_char_class(neg,Class::Upper),
      'a' => {
        if i.next_is(1,'u') {
          new_char_class(neg,Class::UnicodeAlpha)
        }else{
          new_char_class(neg,Class::Alpha)
        }
      },
      c => RegexSymbol::Char(c)
    };
    i.index+=1;
    if let Some(c) = i.get() {
      if c=='}' {
        i.index+=1;
        return Ok(Some(x));
      }
    }
    return syntax_error("expected '}'.");
  }else{
    return syntax_error("expected character.");
  }
}

fn char_class(i: &mut TokenIterator) -> Result<Option<RegexSymbol>,String> {
  let neg = if i.next_is(0,'~') {
    i.index+=1; true
  }else{
    false
  };
  let mut v: Vec<RegexSymbol> = Vec::new();
  loop{
    if i.next_is(1,'-') && i.exists_at(2) {
      let a = i.at(0);
      let b = i.at(2);
      v.push(RegexSymbol::Range(Box::new(Range{a,b})));
      i.index+=3;
    }else if i.exists_at(0) {
      let c = i.at(0);
      if c==']' {i.index+=1; break;}
      v.push(RegexSymbol::Char(c));
      i.index+=1;
    }else{
      return syntax_error("expected ']', but reached end of regex.");
    }
  }
  let y = RegexSymbol::Or(v.into_boxed_slice());
  return Ok(Some(if neg {
    RegexSymbol::Complement(Box::new(y))
  }else{y}));
}

fn atom(i: &mut TokenIterator) -> Result<Option<RegexSymbol>,String> {
  if let Some(c) = i.get() {
    if c == '.' {
      i.index+=1;
      return Ok(Some(RegexSymbol::Dot));
    }else if c == '(' {
      i.index+=1;
      let r = try!(regex(i));
      if let Some(c) = i.get() {
        if c == ')' {
          i.index+=1;
          return Ok(Some(r));
        }else{
          return syntax_error("expected ')'.");
        }
      }else{
        return syntax_error("unexpected end of regex.");
      }
    }else if c == '{' {
      i.index+=1;
      return escape_seq(i);
    }else if c == '[' {
      i.index+=1;
      return char_class(i);
    }else{
      i.index+=1;
      return Ok(Some(RegexSymbol::Char(c)));
    }
  }else{
    return Ok(None);
  }
}

fn postfix_operation(i: &mut TokenIterator) -> Result<Option<RegexSymbol>,String> {
  if let Some(x) = try!(atom(i)) {
    if let Some(c) = i.get() {
      if c == '?' {
        i.index+=1;
        return Ok(Some(RegexSymbol::Qm(Box::new(x))));
      }else if c == '*' {
        i.index+=1;
        return Ok(Some(RegexSymbol::Star(Box::new(x))));
      }else if c == '+' {
        i.index+=1;
        return Ok(Some(RegexSymbol::Plus(Box::new(x))));
      }else{
        return Ok(Some(x));
      }
    }else{
      return Ok(Some(x));
    }
  }else{
    return Ok(None);
  }
}

fn regex_chain(i: &mut TokenIterator) -> Result<RegexSymbol,String> {
  let mut v: Vec<RegexSymbol> = Vec::new();
  loop{
    if let Some(x) = try!(postfix_operation(i)) {
      v.push(x);
    }else{
      break;
    }
    if let Some(c) = i.get() {
      if c==')' || c=='|' {break;}
    }
  }
  if v.len()==1 {
    return Ok(match v.pop() {Some(x) => x, None => unreachable!()});
  }else{
    return Ok(RegexSymbol::Regex(v.into_boxed_slice()));
  }
}

fn or_operation(i: &mut TokenIterator) -> Result<RegexSymbol,String> {
  let mut v: Vec<RegexSymbol> = Vec::new();
  v.push(try!(regex_chain(i)));
  loop{
    if let Some(c) = i.get() {
      if c == '|' {
        i.index+=1;
        v.push(try!(regex_chain(i)));
      }else{
        break;
      }
    }else{
      break;
    }
  }
  if v.len()==1 {
    return Ok(match v.pop() {Some(x) => x, None => unreachable!()});
  }else{
    return Ok(RegexSymbol::Or(v.into_boxed_slice()));
  }
}

fn regex(i: &mut TokenIterator) -> Result<RegexSymbol,String> {
  return or_operation(i);
}

fn compile(s: &Vec<char>) -> Result<RegexSymbol,String> {
  let v = scan(s);
  let mut i = TokenIterator{v: v, index: 0};
  return regex(&mut i);
}

fn symbol_match(regex: &RegexSymbol, i: &mut CharIterator) -> bool {
  match *regex {
    RegexSymbol::Char(c) => {
      if let Some(x) = i.get() {
        i.index+=1;
        return x == c;
      }else{
        return false;
      }
    },
    RegexSymbol::Dot => {
      if let Some(x) = i.get() {
        i.index+=1;
        return true;
      }else{
        return false;
      }
    },
    RegexSymbol::Regex(ref a) => {
      for r in a.iter() {
        if !symbol_match(r,i) {return false;}
      }
      return true;
    },
    RegexSymbol::Qm(ref r) => {
      let index = i.index;
      if !symbol_match(r,i) {
        i.index = index;
      }
      return true;
    },
    RegexSymbol::Star(ref r) => {
      loop{
        let index = i.index;
        if !symbol_match(r,i) {
          i.index = index;
          break;
        }
      }
      return true;
    },
    RegexSymbol::Plus(ref r) => {
      if !symbol_match(r,i) {return false;}
      loop{
        let index = i.index;
        if !symbol_match(r,i) {
          i.index = index;
          break;
        }
      }
      return true;
    },
    RegexSymbol::Or(ref a) => {
      let index = i.index;
      for r in a.iter() {
        if symbol_match(r,i) {return true;}
        i.index = index;
      }
      return false;
    },
    RegexSymbol::Class(char_class) => {
      if let Some(x) = i.get() {
        i.index+=1;
        let y = match char_class.value {
          Class::Alpha => x.is_alphabetic() && x.is_ascii(),
          Class::UnicodeAlpha => x.is_alphabetic(),
          Class::Digit => x.is_digit(10),
          Class::HexDigit => x.is_digit(16),
          Class::Lower => x.is_lowercase() && x.is_ascii(),
          Class::Upper => x.is_uppercase() && x.is_ascii()
        };
        return if char_class.neg {!y} else {y};
      }else{
        return false;
      }
    },
    RegexSymbol::Range(ref range) => {
      if let Some(x) = i.get() {
        i.index+=1;
        return range.a <= x && x <= range.b;
      }else{
        return false;
      }  
    },
    RegexSymbol::Complement(ref compl) => {
      let index = i.index+1;
      if symbol_match(compl,i) {
        i.index = index;
        return false;
      }else{
        i.index = index;
        return true;
      }
    }
  }
}

fn re_match(regex: &RegexSymbol, s: &Vec<char>) -> bool {
  let mut i = CharIterator{index: 0, v: s};
  let y = symbol_match(regex,&mut i);
  return y && i.index == i.v.len();
}

fn regex_match(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  match argv.len() {
    1 => {}, n => return env.argc_error(n,1,1,"match")
  }
  match argv[0] {
    Object::String(ref s) => {
      if let Some(r) = Regex::downcast(pself) {
        Ok(Object::Bool(re_match(&r.regex,&s.v)))
      }else{
        env.type_error("Type error in r.match(s): r is not a regex.")
      }
    },
    ref s => env.type_error1("Type error in r.match(s): s is not a string.","s",s)
  }
}

struct Regex{
  index: usize,
  regex: RegexSymbol
}

impl Regex{
  fn downcast(x: &Object) -> Option<&Self> {
    if let Object::Interface(ref a) = *x {
      a.as_any().downcast_ref::<Self>()
    }else{
      None
    }
  }
}

impl Interface for Regex{
  fn as_any(&self) -> &Any {self}
  fn type_name(&self) -> String {
    "Regex".to_string()
  }
  fn get(&self, key: &Object, env: &mut Env) -> FnResult {
    let t = &env.rte().interface_types.borrow()[self.index];
    match t.get(key) {
      Some(value) => return Ok(value),
      None => env.index_error(&format!(
        "Index error in t.{0}: {0} not found.", key
      ))
    }
  }
}

fn regex_compile(index: usize) -> Object {
  let f = Box::new(move |env: &mut Env, pself: &Object, argv: &[Object]| -> FnResult {
    match argv.len() {
      1 => {}, n => return env.argc_error(n,1,1,"compile")
    }
    match argv[0] {
      Object::String(ref s) => {
        let r = match compile(&s.v) {
          Ok(r) => r,
          Err(e) => {
            return env.std_exception(&e);
          }
        };
        return Ok(Object::Interface(Rc::new(Regex{
          index: index, regex: r
        })));
      },
      ref s => env.type_error1("Type error in compile(s): s is not a string.","s",s)
    }
  });
  return Function::mutable(f,1,1);
}

pub fn load_regex(env: &mut Env) -> Object {
  let type_regex = Table::new(Object::Null);
  {
    let mut m = type_regex.map.borrow_mut();
    m.insert_fn_plain("match",regex_match,1,1);
  }
  let mut v = env.rte().interface_types.borrow_mut();
  let index = v.len();
  v.push(type_regex);

  let regex = new_module("regex");
  {
    let mut m = regex.map.borrow_mut();
    m.insert("compile",regex_compile(index));
  }
  return Object::Table(Rc::new(regex));
}
