
use std::rc::Rc;
use std::any::Any;

use crate::object::{
    Object, List, CharString, Function, FnResult, Interface,
    Exception, new_module, downcast, ptr_eq_plain
};
use crate::vm::{RTE,Env,interface_index,interface_types_set};
use crate::class;

#[derive(Debug,Clone,Copy)]
enum Class{
    Alpha, Digit, HexDigit, Lower, Upper,
    Whitespace, UnicodeAlpha, UnicodeLower, UnicodeUpper
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
    Regex(Box<[RegexSymbol]>), Group(Box<RegexSymbol>),
    Qm(Box<RegexSymbol>),
    Plus(Box<RegexSymbol>),
    Star(Box<RegexSymbol>),
    Or(Box<[RegexSymbol]>)
}

#[allow(dead_code)]
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
    v: &'a [char],
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

fn scan(s: &[char]) -> Vec<Token> {
    let mut i: usize = 0;
    let mut line: usize = 0;
    let mut col: usize = 0;
    let n = s.len();
    let mut v: Vec<Token> = Vec::new();
    while i<n {
        let c = s[i];
        if c==' ' {col+=1; i+=1;}
        else if c=='\n' {line+=1; col=0;i+=1;}
        else{
            v.push(Token {c, index: i, line, col});
            col+=1;
            i+=1;
        }
    }
    return v;
}

fn syntax_error(text: &str) -> Result<Option<RegexSymbol>,String> {
    Err(format!("Syntax error in regex: {}", text))
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
            'a' => new_char_class(neg,Class::Alpha),
            '_' => new_char_class(neg,Class::Whitespace),
            'u' => {
                if i.next_is(1,'a') {
                    i.index+=1;
                    new_char_class(neg,Class::UnicodeAlpha)
                }else if i.next_is(1,'l') {
                    i.index+=1;
                    new_char_class(neg,Class::UnicodeLower)
                }else if i.next_is(1,'u') {
                    i.index+=1;
                    new_char_class(neg,Class::UnicodeUpper)
                }else{
                    new_char_class(neg,Class::Upper)
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
        if i.exists_at(0) {
            let c = i.at(0);
            if c=='{' {
                i.index+=1;
                v.push(match escape_seq(i)? {
                    Some(x) => x,
                    None => unreachable!()
                });
            }else if i.next_is(1,'-') && i.exists_at(2) && !i.next_is(2,']') {
                let a = i.at(0);
                let b = i.at(2);
                v.push(RegexSymbol::Range(Box::new(Range{a,b})));
                i.index+=3;
            }else{
                if c==']' {i.index+=1; break;}
                v.push(RegexSymbol::Char(c));
                i.index+=1;
            }
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
            let c = i.get().unwrap_or('_');
            let r = if c == '*' {
                i.index+=1;
                RegexSymbol::Group(Box::new(regex(i)?))
            }else{
                regex(i)?
            };
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
    if let Some(x) = atom(i)? {
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
        if let Some(x) = postfix_operation(i)? {
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
    v.push(regex_chain(i)?);
    while let Some(c) = i.get() {
        if c == '|' {
            i.index+=1;
            v.push(regex_chain(i)?);
        }else{
            break;
        }
    }
    return Ok(if v.len()==1 {
        match v.pop() {Some(x) => x, None => unreachable!()}
    }else{
        RegexSymbol::Or(v.into_boxed_slice())
    });
}

fn regex(i: &mut TokenIterator) -> Result<RegexSymbol,String> {
    return or_operation(i);
}

fn compile(s: &[char]) -> Result<RegexSymbol,String> {
    let v = scan(s);
    let mut i = TokenIterator{v, index: 0};
    return regex(&mut i);
}

fn symbol_match(regex: &RegexSymbol, i: &mut CharIterator,
    groups: &mut Option<Vec<Object>>
) -> bool {
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
            if i.get().is_some() {
                i.index+=1;
                return true;
            }else{
                return false;
            }
        },
        RegexSymbol::Regex(ref a) => {
            for r in a.iter() {
                if !symbol_match(r,i,groups) {return false;}
            }
            return true;
        },
        RegexSymbol::Group(ref r) => {
            let j = i.index;
            if symbol_match(r,i,groups) {
                if let Some(ref mut groups) = *groups {
                    let x = CharString::new_object(i.v[j..i.index].to_vec());
                    groups.push(x);
                }
                return true;
            }else{
                return false;
            }
        },
        RegexSymbol::Qm(ref r) => {
            let index = i.index;
            if !symbol_match(r,i,groups) {
                i.index = index;
            }
            return true;
        },
        RegexSymbol::Star(ref r) => {
            loop{
                let index = i.index;
                if !symbol_match(r,i,groups) {
                    i.index = index;
                    break;
                }
            }
            return true;
        },
        RegexSymbol::Plus(ref r) => {
            if !symbol_match(r,i,groups) {return false;}
            loop{
                let index = i.index;
                if !symbol_match(r,i,groups) {
                    i.index = index;
                    break;
                }
            }
            return true;
        },
        RegexSymbol::Or(ref a) => {
            let index = i.index;
            for r in a.iter() {
                if symbol_match(r,i,groups) {return true;}
                i.index = index;
            }
            return false;
        },
        RegexSymbol::Class(char_class) => {
            if let Some(x) = i.get() {
                i.index+=1;
                let y = match char_class.value {
                    Class::Alpha => x.is_ascii_alphabetic(),
                    Class::Digit => x.is_digit(10),
                    Class::HexDigit => x.is_digit(16),
                    Class::Lower => x.is_ascii_lowercase(),
                    Class::Upper => x.is_ascii_uppercase(),
                    Class::Whitespace => x.is_whitespace(),
                    Class::UnicodeAlpha => x.is_alphabetic(),
                    Class::UnicodeLower => x.is_lowercase(),
                    Class::UnicodeUpper => x.is_uppercase()
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
            if i.get().is_some() {
                let index = i.index+1;
                if symbol_match(compl,i,groups) {
                    i.index = index;
                    return false;
                }else{
                    i.index = index;
                    return true;
                }
            }else{
                return false;
            }
        }
    }
}

fn re_match(regex: &RegexSymbol, s: &[char]) -> bool {
    let mut i = CharIterator{index: 0, v: s};
    let y = symbol_match(regex,&mut i,&mut None);
    return y && i.index == i.v.len();
}

fn re_groups(regex: &RegexSymbol, s: &[char]) -> Object {
    let mut groups: Option<Vec<Object>> = Some(Vec::new());
    let mut i = CharIterator{index: 0, v: s};
    let y = symbol_match(regex,&mut i,&mut groups);
    if y && i.index == i.v.len() {
        return List::new_object(groups.unwrap());
    }else{
        return Object::Null;
    }
}


fn re_list(regex: &RegexSymbol, s: &[char]) -> Object {
    let n = s.len();
    let mut i = 0;
    let mut j = CharIterator{index: 0, v: s};
    let mut a: Vec<Object> = Vec::new();
    while i<n {
        j.index = i;
        if symbol_match(regex,&mut j,&mut None) {
            let x = CharString::new_object(j.v[i..j.index].to_vec());
            a.push(x);
            i = j.index;
        }else{
            i+=1;
        }
    }
    return List::new_object(a);
}

fn re_split(regex: &RegexSymbol, s: &[char]) -> Object {
    let n = s.len();
    let mut i = 0;
    let mut j = CharIterator{index: 0, v: s};
    let mut a: Vec<Object> = Vec::new();
    let mut start = 0;
    while i<n {
        j.index = i;
        if symbol_match(regex,&mut j,&mut None) {
            let x = CharString::new_object(j.v[start..i].to_vec());
            a.push(x);
            i = j.index;
            start = j.index;
        }else{
            i+=1;
        }
    }
    let x = CharString::new_object(j.v[start..].to_vec());
    a.push(x);
    return List::new_object(a);
}

fn re_replace(regex: &RegexSymbol, s: &[char],
    env: &mut Env, f: &Object
) -> FnResult
{
    let n = s.len();
    let mut i = 0;
    let mut j = CharIterator{index: 0, v: s};
    let mut buffer: Vec<char> = Vec::with_capacity(n);
    while i<n {
        j.index = i;
        if symbol_match(regex,&mut j,&mut None) {
            let x = CharString::new_object(j.v[i..j.index].to_vec());
            let y = env.call(f,&Object::Null,&[x])?;
            match y {
                Object::String(sy) => {
                    buffer.extend_from_slice(&sy.data);
                },
                _ => return env.type_error(
                    "Type error in r.replace(s,f): f(x) is not a string.")
            }
            i = j.index;
        }else{
            buffer.push(j.v[i]);
            i+=1;
        }
    }
    return Ok(CharString::new_object(buffer));
}

fn regex_match(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"match")
    }
    match argv[0] {
        Object::String(ref s) => {
            if let Some(r) = downcast::<Regex>(pself) {
                Ok(Object::Bool(re_match(&r.regex,&s.data)))
            }else{
                env.type_error("Type error in r.match(s): r is not a regex.")
            }
        },
        ref s => env.type_error1("Type error in r.match(s): s is not a string.","s",s)
    }
}

fn regex_list(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"list")
    }
    match argv[0] {
        Object::String(ref s) => {
            if let Some(r) = downcast::<Regex>(pself) {
                Ok(re_list(&r.regex,&s.data))
            }else{
                env.type_error1("Type error in r.list(s): r is not a regex.","r",pself)
            }
        },
        ref s => env.type_error1("Type error in r.list(s): s is not a string.","s",s)
    }
}

fn regex_split(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"split")
    }
    match argv[0] {
        Object::String(ref s) => {
            if let Some(r) = downcast::<Regex>(pself) {
                Ok(re_split(&r.regex,&s.data))
            }else{
                env.type_error1("Type error in r.split(s): r is not a regex.","r",pself)
            }
        },
        ref s => env.type_error1("Type error in r.split(s): s is not a string.","s",s)
    }
}

fn regex_groups(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"groups")
    }
    match argv[0] {
        Object::String(ref s) => {
            if let Some(r) = downcast::<Regex>(pself) {
                Ok(re_groups(&r.regex,&s.data))
            }else{
                env.type_error1("Type error in r.groups(s): r is not a regex.","r",pself)
            }
        },
        ref s => env.type_error1("Type error in r.groups(s): s is not a string.","s",s)
    }
}

fn regex_replace(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        2 => {}, n => return env.argc_error(n,2,2,"replace")
    }
    match argv[0] {
        Object::String(ref s) => {
            if let Some(r) = downcast::<Regex>(pself) {
                re_replace(&r.regex,&s.data,env,&argv[1])
            }else{
                env.type_error1("Type error in r.replace(s,f): r is not a regex.","r",pself)
            }
        },
        ref s => env.type_error1("Type error in r.replace(s,f): s is not a string.","s",s)
    }
}

struct Regex{
    index: usize,
    regex: RegexSymbol
}

impl Interface for Regex{
    fn as_any(&self) -> &dyn Any {self}
    fn type_name(&self, _env: &mut Env) -> String {
        "Regex".to_string()
    }
    fn to_string(self: Rc<Self>, _env: &mut Env) -> Result<String,Box<Exception>> {
        Ok("regex object".to_string())
    }
    fn get_type(&self, env: &mut Env) -> FnResult {
        Ok(Object::Interface(env.rte().interface_types
            .borrow()[self.index].clone()))
    }
    fn is_instance_of(&self, type_obj: &Object, rte: &RTE) -> bool {
        if let Object::Interface(p) = type_obj {
            ptr_eq_plain(p,&rte.interface_types.borrow()[self.index])
        }else{false}
    }
    fn get(self: Rc<Self>, key: &Object, env: &mut Env) -> FnResult {
        let t = &env.rte().interface_types.borrow()[self.index];
        match t.slot(key) {
            Some(value) => return Ok(value),
            None => env.index_error(&format!(
                "Index error in t.{0}: {0} not found.", key
            ))
        }
    }
}

fn regex_compile(index: usize) -> Object {
    let f = Box::new(move |env: &mut Env, _pself: &Object, argv: &[Object]| -> FnResult {
        match argv.len() {
            1 => {}, n => return env.argc_error(n,1,1,"re")
        }
        match argv[0] {
            Object::String(ref s) => {
                let r = match compile(&s.data) {
                    Ok(r) => r,
                    Err(e) => return env.std_exception(&e)
                };
                return Ok(Object::Interface(Rc::new(Regex {
                    index, regex: r
                })));
            },
            ref s => env.type_error1("Type error in re(s): s is not a string.","s",s)
        }
    });
    return Function::mutable(f,1,1);
}

pub fn load_regex(env: &mut Env) -> Object {
    let type_regex = class::Class::new("Regex",&Object::Null);
    {
        let mut m = type_regex.map.borrow_mut();
        m.insert_fn_plain("match",regex_match,1,1);
        m.insert_fn_plain("list",regex_list,1,1);
        m.insert_fn_plain("split",regex_split,1,1);
        m.insert_fn_plain("groups",regex_groups,1,1);
        m.insert_fn_plain("replace",regex_replace,2,2);
    }
    interface_types_set(env.rte(),interface_index::REGEX,type_regex.clone());

    let regex = new_module("regex");
    {
        let mut m = regex.map.borrow_mut();
        m.insert("re",regex_compile(interface_index::REGEX));
        m.insert("Regex",Object::Interface(type_regex));
    }
    return Object::Interface(Rc::new(regex));
}
