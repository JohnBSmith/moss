
use std::rc::Rc;
use std::collections::HashMap;
use std::mem::replace;
use std::str::Chars;
use std::char;
use system;
use vm::{bc, BCSIZE, BCASIZE, BCAASIZE, Module, RTE};
use object::{Object, CharString, VARIADIC};

#[derive(Copy,Clone,PartialEq)]
pub enum Value{
    None, Optional, Null, Empty
}

#[derive(Copy,Clone,PartialEq)]
enum SymbolType{
    None, Operator, Separator, Bracket,
    Bool, Int, Float, Imag,
    String, Identifier, Keyword,
    Assignment
}

#[derive(Copy,Clone,PartialEq)]
enum Symbol{
    None, Plus, Minus, Ast, Div, Idiv, Mod, Pow,
    Lt, Gt, Le, Ge, Eq, Ne, In, Is, Isin, Notin, Isnot, Range,
    And, Or, Amp, Vline, Neg, Not, Tilde, Svert, Assignment,
    PLeft, PRight, BLeft, BRight, CLeft, CRight, Newline,
    Lshift, Rshift, Assert, Begin, Break, Catch, Continue,
    Elif, Else, End, For, Global, Goto, Label, Of,
    If, While, Do, Raise, Return, Fn, Function, Table, Then, Try,
    Use, Yield, True, False, Null, Dot, Comma, Colon, Semicolon,
    List, Map, Application, Index, Block, Statement, Terminal,
    APlus, AMinus, AAst, ADiv, AIdiv, AMod, AAmp, AVline, ASvert,
    Empty, Tuple, Splat
}

enum Item{
    None,
    String(String),
    Int(i32),
    // Float(f64)
}

impl Item {
    fn assert_string(&self) -> &String {
        match *self {Item::String(ref s) => s, _ => unreachable!()}
    }
    fn _assert_int(&self) -> i32 {
        match *self {Item::Int(x) => x, _ => unreachable!()}
    }
}

pub struct Token {
    token_type: SymbolType,
    value: Symbol,
    line: usize,
    col: usize,
    item: Item
}

impl Token {
    fn operator(line: usize, col: usize, value: Symbol) -> Token {
        Token{
            token_type: SymbolType::Operator, value: value,
            col: col, line: line, item: Item::None
        }
    }

    fn aoperator(line: usize, col: usize, value: Symbol) -> Token {
        Token{
            token_type: SymbolType::Assignment, value: value,
            col: col, line: line, item: Item::None
        }
    }

    fn bracket(line: usize, col: usize, value: Symbol) -> Token {
        Token{
            token_type: SymbolType::Bracket, value: value,
            col: col, line: line, item: Item::None
        }
    }
}

struct KeywordsElement {
    s: &'static str,
    t: &'static SymbolType,
    v: &'static Symbol
}

static KEYWORDS: &'static [KeywordsElement] = &[
      KeywordsElement{s: "assert",  t: &SymbolType::Keyword, v: &Symbol::Assert},
      KeywordsElement{s: "and",     t: &SymbolType::Operator,v: &Symbol::And},
      KeywordsElement{s: "begin",   t: &SymbolType::Keyword, v: &Symbol::Begin},
      KeywordsElement{s: "break",   t: &SymbolType::Keyword, v: &Symbol::Break},
      KeywordsElement{s: "catch",   t: &SymbolType::Keyword, v: &Symbol::Catch},
      KeywordsElement{s: "continue",t: &SymbolType::Keyword, v: &Symbol::Continue},
      KeywordsElement{s: "do",      t: &SymbolType::Keyword, v: &Symbol::Do},
      KeywordsElement{s: "elif",    t: &SymbolType::Keyword, v: &Symbol::Elif},
      KeywordsElement{s: "else",    t: &SymbolType::Keyword, v: &Symbol::Else},
      KeywordsElement{s: "end",     t: &SymbolType::Keyword, v: &Symbol::End},
      KeywordsElement{s: "false",   t: &SymbolType::Bool,    v: &Symbol::False},
      KeywordsElement{s: "for",     t: &SymbolType::Keyword, v: &Symbol::For},
      KeywordsElement{s: "fn",      t: &SymbolType::Keyword, v: &Symbol::Fn},
      KeywordsElement{s: "function",t: &SymbolType::Keyword, v: &Symbol::Function},
      KeywordsElement{s: "global",  t: &SymbolType::Keyword, v: &Symbol::Global},
      KeywordsElement{s: "goto",    t: &SymbolType::Keyword, v: &Symbol::Goto},
      KeywordsElement{s: "label",   t: &SymbolType::Keyword, v: &Symbol::Label},
      KeywordsElement{s: "if",      t: &SymbolType::Keyword, v: &Symbol::If},
      KeywordsElement{s: "in",      t: &SymbolType::Operator,v: &Symbol::In},
      KeywordsElement{s: "is",      t: &SymbolType::Operator,v: &Symbol::Is},
      KeywordsElement{s: "not",     t: &SymbolType::Operator,v: &Symbol::Not},
      KeywordsElement{s: "null",    t: &SymbolType::Keyword, v: &Symbol::Null},
      KeywordsElement{s: "of",      t: &SymbolType::Keyword, v: &Symbol::Of},
      KeywordsElement{s: "or",      t: &SymbolType::Operator,v: &Symbol::Or},
      KeywordsElement{s: "public",  t: &SymbolType::Keyword, v: &Symbol::Global},
      KeywordsElement{s: "raise",   t: &SymbolType::Keyword, v: &Symbol::Raise},
      KeywordsElement{s: "return",  t: &SymbolType::Keyword, v: &Symbol::Return},
      KeywordsElement{s: "table",   t: &SymbolType::Keyword, v: &Symbol::Table},
      KeywordsElement{s: "then",    t: &SymbolType::Keyword, v: &Symbol::Then},
      KeywordsElement{s: "true",    t: &SymbolType::Bool,    v: &Symbol::True},
      KeywordsElement{s: "try",     t: &SymbolType::Keyword, v: &Symbol::Try},
      KeywordsElement{s: "use",     t: &SymbolType::Keyword, v: &Symbol::Use},
      KeywordsElement{s: "while",   t: &SymbolType::Keyword, v: &Symbol::While},
      KeywordsElement{s: "yield",   t: &SymbolType::Keyword, v: &Symbol::Yield}
];

pub struct SyntaxError {
    line: usize, col: usize,
    file: String, s: String
}

pub enum EnumError{
    Syntax(SyntaxError)
}
type Error = Box<EnumError>;

pub fn print_syntax_error(e: &SyntaxError){
    println!("Line {}, col {} ({}):",e.line,e.col,e.file);
    println!("Syntax error: {}",e.s);
}

pub fn print_error(e: &Error){
    match **e {
        EnumError::Syntax(ref e) => {
            print_syntax_error(e);
        }
    }
}

#[allow(dead_code)]
fn compiler_error() -> !{
    unreachable!("compiler error");
}

fn is_keyword(id: &String) -> Option<&'static KeywordsElement> {
    let n: usize = KEYWORDS.len();
    for i in 0..n {
        if KEYWORDS[i].s==id  {return Some(&KEYWORDS[i]);}
    }
    return None;
}

pub fn scan(s: &str, line_start: usize, file: &str, new_line_start: bool)
    -> Result<Vec<Token>,Error>
{
    let mut v: Vec<Token> = Vec::new();
    if new_line_start {
        v.push(Token{token_type: SymbolType::Separator, value: Symbol::Newline,
            line: line_start, col: 1, item: Item::None});
    }

    let mut line = line_start;
    let mut col = 1;
    let mut hcol: usize;

    let a: Vec<char> = s.chars().collect();
    let mut i = 0;
    let n = a.len();
    while i<n {
        let c = a[i];
        if c.is_digit(10) {
            let j = i;
            hcol = col;
            let mut token_type = SymbolType::Int;
            while i<n {
                if a[i].is_digit(10) {
                    i+=1; col+=1;
                }else if a[i]=='.' && token_type != SymbolType::Float{
                    if i+1<n && a[i+1]=='.' {break;}
                    i+=1; col+=1;
                    token_type = SymbolType::Float;
                }else if a[i]=='i' {
                    token_type = SymbolType::Imag;
                    break;
                }else if a[i]=='e' || a[i]=='E' {
                    i+=1; col+=1;
                    token_type = SymbolType::Float;
                    if i<n && (a[i]=='+' || a[i]=='-') {
                        i+=1; col+=1;
                    }
                }else{
                    break;
                }
            }
            let number: &String = &a[j..i].iter().cloned().collect();
            if token_type == SymbolType::Int {
                match number.parse::<i32>() {
                    Ok(x) => {
                        v.push(Token{token_type: token_type, value: Symbol::None,
                            line: line, col: hcol, item: Item::Int(x)});
                    },
                    Err(_) => {
                        v.push(Token{token_type: token_type, value: Symbol::None,
                            line: line, col: hcol, item: Item::String(number.clone())});
                    }
                };
            }else{
                if token_type == SymbolType::Imag {
                    i+=1; col+=1;
                }
                v.push(Token{token_type: token_type, value: Symbol::None,
                    line: line, col: hcol, item: Item::String(number.clone())});
            }
        }else if (c.is_alphabetic() && c.is_ascii()) || a[i]=='_' {
            let j = i;
            hcol = col;
            while i<n && (a[i].is_alphabetic() || a[i].is_digit(10) || a[i]=='_') {
                i+=1; col+=1;
            }
            let id: &String = &a[j..i].iter().cloned().collect();
            match is_keyword(id) {
                Some(x) => {
                    if *x.v==Symbol::In {
                        let len = v.len();
                        if len>0 {
                            if v[len-1].value == Symbol::Not {
                                v[len-1].value = Symbol::Notin;
                                continue;
                            }else if v[len-1].value == Symbol::Is {
                                v[len-1].value = Symbol::Isin;
                                continue;
                            }
                        }
                    }
                    v.push(Token{token_type: *x.t, value: *x.v,
                        line: line, col: hcol, item: Item::None});
                },
                None => {
                    v.push(Token{token_type: SymbolType::Identifier,
                        value: Symbol::None, line: line, col: hcol,
                        item: Item::String(id.clone())});
                }
            }
        }else{
            match c {
                ' ' | '\t' => {
                    i+=1; col+=1;
                },
                '\n' => {
                    v.push(Token{token_type: SymbolType::Separator,
                        value: Symbol::Newline, line: line, col: col,
                        item: Item::None});
                    i+=1; col=1; line+=1;
                },
                ',' => {
                    v.push(Token{token_type: SymbolType::Separator,
                        value: Symbol::Comma, line: line, col: col,
                        item: Item::None});
                    i+=1; col+=1;
                },
                ':' => {
                    if i+1<n && a[i+1]=='=' {
                        v.push(Token::operator(line,col,Symbol::Assignment));
                        i+=2; col+=2;
                    }else{
                        v.push(Token{token_type: SymbolType::Separator,
                            value: Symbol::Colon, line: line, col: col,
                            item: Item::None});
                        i+=1; col+=1;
                    }
                },
                ';' => {
                    v.push(Token{token_type: SymbolType::Separator,
                        value: Symbol::Semicolon, line: line, col: col,
                        item: Item::None});
                    i+=1; col+=1;
                },
                '(' => {
                    v.push(Token::bracket(line,col,Symbol::PLeft));
                    i+=1; col+=1;
                },
                ')' => {
                    v.push(Token::bracket(line,col,Symbol::PRight));
                    i+=1; col+=1;
                },
                '[' => {
                    v.push(Token::bracket(line,col,Symbol::BLeft));
                    i+=1; col+=1;
                },
                ']' => {
                    v.push(Token::bracket(line,col,Symbol::BRight));
                    i+=1; col+=1;
                },
                '{' => {
                    v.push(Token::bracket(line,col,Symbol::CLeft));
                    i+=1; col+=1;
                },
                '}' => {
                    v.push(Token::bracket(line,col,Symbol::CRight));
                    i+=1; col+=1;
                },
                '=' => {
                    if i+1<n && a[i+1]=='=' {
                        v.push(Token::operator(line,col,Symbol::Eq));
                        i+=2; col+=2;
                    }else{
                        v.push(Token::operator(line,col,Symbol::Assignment));
                        i+=1; col+=1;
                    }
                },
                '+' => {
                    if i+1<n && a[i+1]=='=' {
                        v.push(Token::aoperator(line,col,Symbol::APlus));
                        i+=2; col+=2;
                    }else{
                        v.push(Token::operator(line,col,Symbol::Plus));
                        i+=1; col+=1;
                    }
                },
                '-' => {
                    if i+1<n && a[i+1]=='=' {
                        v.push(Token::aoperator(line,col,Symbol::AMinus));
                        i+=2; col+=2;
                    }else{
                        v.push(Token::operator(line,col,Symbol::Minus));
                        i+=1; col+=1;
                    }
                },
                '*' => {
                    if i+1<n && a[i+1]=='=' {
                        v.push(Token::aoperator(line,col,Symbol::AAst));
                        i+=2; col+=2;
                    }else{
                        v.push(Token::operator(line,col,Symbol::Ast));
                        i+=1; col+=1;
                    }
                },
                '/' => {
                    if i+1<n && a[i+1]=='*' {
                        i+=2; col+=2;
                        while i+1<n {
                            if a[i]=='*' && a[i+1]=='/' {i+=2; col+=2; break;}
                            if a[i]=='\n' {col=1; line+=1;} else{col+=1;}
                            i+=1;
                        }
                    }else if i+1<n && a[i+1]=='/' {
                        if i+2<n && a[i+2]=='=' {
                            v.push(Token::aoperator(line,col,Symbol::AIdiv));
                            i+=3; col+=3;
                        }else{
                            v.push(Token::operator(line,col,Symbol::Idiv));
                            i+=2; col+=2;
                        }
                    }else if i+1<n && a[i+1]=='=' {
                        v.push(Token::aoperator(line,col,Symbol::ADiv));
                        i+=2; col+=2;
                    }else{
                        v.push(Token::operator(line,col,Symbol::Div));
                        i+=1; col+=1;
                    }
                },
                '%' => {
                    if i+1<n && a[i+1]=='=' {
                        v.push(Token::aoperator(line,col,Symbol::AMod));
                        i+=2; col+=2;
                    }else{
                        v.push(Token::operator(line,col,Symbol::Mod));
                        i+=1; col+=1;
                    }
                },
                '^' => {
                    v.push(Token::operator(line,col,Symbol::Pow));
                    i+=1; col+=1;
                },
                '.' => {
                    if i+1<n && a[i+1]=='.' {
                        if v.len()>0 && v[v.len()-1].value==Symbol::BLeft {
                            v.push(Token{token_type: SymbolType::Keyword,
                                value: Symbol::Null, line: line, col: col,
                                item: Item::None});
                        }
                        v.push(Token::operator(line,col,Symbol::Range));
                        i+=2; col+=2;
                    }else{
                        v.push(Token::operator(line,col,Symbol::Dot));
                        i+=1; col+=1;
                    }
                },
                '<' => {
                    if i+1<n && a[i+1]=='=' {
                        v.push(Token::operator(line,col,Symbol::Le));
                        i+=2; col+=2;
                    }else if i+1<n && a[i+1]=='<' {
                        v.push(Token::operator(line,col,Symbol::Lshift));
                        i+=2; col+=2;
                    }else{
                        v.push(Token::operator(line,col,Symbol::Lt));
                        i+=1; col+=1;
                    }
                },
                '>' => {
                    if i+1<n && a[i+1]=='=' {
                        v.push(Token::operator(line,col,Symbol::Ge));
                        i+=2; col+=2;
                    }else if i+1<n && a[i+1]=='>' {
                        v.push(Token::operator(line,col,Symbol::Rshift));
                        i+=2; col+=2;            
                    }else{
                        v.push(Token::operator(line,col,Symbol::Gt));
                        i+=1; col+=1;
                    }
                },
                '|' => {
                    if i+1<n && a[i+1]=='=' {
                        v.push(Token::aoperator(line,col,Symbol::AVline));
                        i+=2; col+=2;
                    }else{
                        v.push(Token::operator(line,col,Symbol::Vline));
                        i+=1; col+=1;
                    }
                },
                '&' => {
                    if i+1<n && a[i+1]=='=' {
                        v.push(Token::aoperator(line,col,Symbol::AAmp));
                        i+=2; col+=2;
                    }else{
                        v.push(Token::operator(line,col,Symbol::Amp));
                        i+=1; col+=1;
                    }
                },
                '$' => {
                    if i+1<n && a[i+1]=='=' {
                        v.push(Token::aoperator(line,col,Symbol::ASvert));
                        i+=2; col+=2;
                    }else{
                        v.push(Token::operator(line,col,Symbol::Svert));
                        i+=1; col+=1;
                    }
                },
                '~' => {
                    v.push(Token::operator(line,col,Symbol::Tilde));
                    i+=1; col+=1;
                },
                '"' => {
                    hcol = col;
                    i+=1; col+=1;
                    if i+1<n && a[i]=='"' && a[i+1]=='"' {
                        i+=2; col+=2;
                        let j=i;
                        while i+2<n && (a[i]!='"' || a[i+1]!='"' || a[i+2]!='"') {
                            if a[i]=='\n' {line+=1; col=0;}
                            i+=1; col+=1;
                        }
                        let s: &String = &a[j..i].iter().cloned().collect();
                        v.push(Token{token_type: SymbolType::String,
                            value: Symbol::None, line: line, col: hcol,
                            item: Item::String(s.clone())
                        });
                        i+=3; col+=3;
                    }else{
                        let j=i;
                        while i<n && a[i]!='"' {
                            if a[i]=='\n' {line+=1; col=0;}
                            i+=1; col+=1;
                        }
                        let s: &String = &a[j..i].iter().cloned().collect();
                        v.push(Token{token_type: SymbolType::String,
                            value: Symbol::None, line: line, col: hcol,
                            item: Item::String(s.clone())
                        });
                        i+=1; col+=1;
                    }
                },
                '\'' => {
                    hcol = col;
                    i+=1; col+=1;
                    let j = i;
                    while i<n && a[i]!='\'' {i+=1; col+=1;}
                    let s: &String = &a[j..i].iter().cloned().collect();
                    v.push(Token{token_type: SymbolType::String,
                        value: Symbol::None, line: line, col: hcol,
                        item: Item::String(s.clone())
                    });
                    i+=1; col+=1;
                },
                '!' => {
                    if i+1<n && a[i+1]=='=' {
                        v.push(Token::operator(line,col,Symbol::Ne));
                        i+=2; col+=2;
                    }else{
                        return Err(Box::new(EnumError::Syntax(SyntaxError{
                            line: line, col: col, file: String::from(file),
                            s: format!("unexpected character '{}'.", c)
                        })));
                    }
                },
                '#' => {
                    while i<n && a[i]!='\n' {i+=1; col+=1;}
                    v.push(Token{token_type: SymbolType::Separator,
                        value: Symbol::Newline, line: line, col: col,
                        item: Item::None});
                    i+=1; col=1; line+=1;
                },
                _ => {
                    return Err(Box::new(EnumError::Syntax(SyntaxError{
                        line: line, col: col, file: String::from(file),
                        s: format!("unexpected character '{}'.", c)
                    })));
                }
            }
        }
    }
    v.push(Token{
        token_type: SymbolType::Separator,
        value: Symbol::Terminal,
        line: line, col: col, item: Item::None
    });
    return Ok(v);
}

fn symbol_to_string(value: Symbol) -> &'static str {
    return match value {
        Symbol::None => "none",
        Symbol::Plus => "+",  Symbol::Minus => "-",
        Symbol::Ast  => "*",  Symbol::Div => "/",
        Symbol::Mod  => "%",  Symbol::Pow => "^",
        Symbol::Vline=> "|",  Symbol::Amp => "&",
        Symbol::Idiv => "//", Symbol::Svert=> "$",
        Symbol::Neg  => "u-", Symbol::Tilde=> "u~",
        Symbol::In   => "in", Symbol::Is => "is",
        Symbol::Isin=>"is in",Symbol::Notin=> "not in",
        Symbol::And  => "and",Symbol::Or => "or",
        Symbol::Not  => "not",Symbol::Isnot => "is not",
        Symbol::PLeft => "(", Symbol::PRight => ")",
        Symbol::BLeft => "[", Symbol::BRight => "]",
        Symbol::CLeft => "{", Symbol::CRight => "}",
        Symbol::Lt    => "<", Symbol::Gt => ">",
        Symbol::Le   => "<=", Symbol::Ge => ">=",
        Symbol::Lshift=>"<<", Symbol::Rshift=> ">>",
        Symbol::Dot   => ".", Symbol::Comma => ",",
        Symbol::Colon => ":", Symbol::Semicolon => ";",
        Symbol::Eq   => "==", Symbol::Ne => "!=",
        Symbol::List => "[]", Symbol::Application => "app",
        Symbol::Map  => "{}", Symbol::Index => "index",
        Symbol::Tuple => "()", Symbol::Splat => "u*",
        Symbol::APlus => "+=",
        Symbol::AMinus => "-=",
        Symbol::AAst => "*=",
        Symbol::ADiv => "/=",
        Symbol::AIdiv => "//=",
        Symbol::AMod => "%=",
        Symbol::AVline => "|=",
        Symbol::AAmp => "&=",
        Symbol::ASvert => "$=",
        Symbol::Block => "block",
        Symbol::Statement => "statement",
        Symbol::Empty => "empty",
        Symbol::Range => "..",
        Symbol::Assignment => "=",
        Symbol::Newline => "\\n",
        Symbol::Assert => "assert",
        Symbol::Begin => "begin",
        Symbol::Break => "break",
        Symbol::Catch => "catch",
        Symbol::Continue => "continue",
        Symbol::Do => "do",
        Symbol::Elif => "elif",
        Symbol::Else => "else",
        Symbol::End => "end",
        Symbol::False => "false",
        Symbol::For => "for",
        Symbol::Global => "global",
        Symbol::Goto => "goto",
        Symbol::If => "if",
        Symbol::Label => "label",
        Symbol::Null => "null",
        Symbol::Of => "of",
        Symbol::Raise => "raise",
        Symbol::Return => "return",
        Symbol::Fn => "fn",
        Symbol::Function => "Function",
        Symbol::Table => "table",
        Symbol::Then => "then",
        Symbol::True => "true",
        Symbol::Try => "try",
        Symbol::Use => "use",
        Symbol::While => "while",
        Symbol::Yield => "yield",
        Symbol::Terminal => "terminal"
    };
}

#[allow(dead_code)]
fn print_token(x: &Token){
    match x.token_type {
        SymbolType::None => {
            print!("[none]");
        },
        SymbolType::Identifier | SymbolType::Int | SymbolType::Float => {
            match x.item {
                Item::String(ref s) => {
                    print!("[{}]",s);
                },
                Item::Int(i) => {
                    print!("[{}]",i);
                },
                _ => compiler_error()
            }
        },
        SymbolType::Imag => {
            print!("[{}i]",match x.item {Item::String(ref s) => s, _ => compiler_error()});    
        },
        SymbolType::String => {
            print!("[\"{}\"]",match x.item {Item::String(ref s) => s, _ => compiler_error()});    
        },
        SymbolType::Operator | SymbolType::Separator |
        SymbolType::Bracket  | SymbolType::Keyword | SymbolType::Bool |
        SymbolType::Assignment => {
            print!("[{}]",symbol_to_string(x.value));
        }
    }
}

#[allow(dead_code)]
pub fn print_vtoken(v: &Vec<Token>){
    for x in v {print_token(x);}
    println!();
}

#[allow(dead_code)]
fn print_ast(t: &AST, indent: usize){
    print!("{:1$}","",indent);
    match t.symbol_type {
        SymbolType::None => {
            println!("none");
        },
        SymbolType::Int => {
            match t.info {
                Info::Int(x) => {println!("{}",x);},
                Info::Long => {
                    println!("long {}",match t.s {Some(ref s) => s, None => compiler_error()});
                },
                _ => compiler_error()
            };
        },
        SymbolType::Identifier | SymbolType::Float => {
            println!("{}",match t.s {Some(ref s) => s, None => compiler_error()});
        },
        SymbolType::Imag => {
            println!("{}i",match t.s {Some(ref s) => s, None => compiler_error()});    
        },
        SymbolType::String => {
            println!("\"{}\"",match t.s {Some(ref s) => s, None => compiler_error()});    
        },
        SymbolType::Operator | SymbolType::Separator |
        SymbolType::Keyword  | SymbolType::Bool | SymbolType::Assignment => {
            if t.value == Symbol::Fn {
                match t.s {
                    Some(ref s) => {println!("fn {}",s);},
                    None => {println!("fn");}
                }
            }else{
                println!("{}",symbol_to_string(t.value));
            }
        },
        _ => {compiler_error();}
    }
    match t.a {
        Some(ref a) => {
            for i in 0..a.len() {
                print_ast(&a[i],indent+2);
            }
        },
        None => {}
    };
}

fn scan_line(line_start: usize, h: &mut system::History, new_line_start: bool) -> Result<Vec<Token>,Error>{
    let input = match system::getline_history("| ",h) {
        Ok(x) => x,
        Err(_) => panic!()
    };
    h.append(&input);
    return scan(&input,line_start,"command line",new_line_start);
}

enum ComplexInfoA{
}

enum Info{
    None, SelfArg, Coroutine, _A(Box<ComplexInfoA>),
    Int(i32), ArgvInfo{variadic: bool, selfarg: bool},
    Long
}

struct AST{
    line: usize, col: usize,
    symbol_type: SymbolType,
    value: Symbol,
    info: Info,
    s: Option<String>,
    a: Option<Box<[Rc<AST>]>>
}

#[derive(Clone,Copy,PartialEq)]
enum VarType{
    Local, Argument, Context, Global, FnId
}

struct VarInfo{
    s: String,
    index: usize,
    var_type: VarType
}

struct VarTab{
    v: Vec<VarInfo>,
    count_local: usize,
    count_arg: usize,
    count_context: usize,
    count_global: usize,
    context: Option<Box<VarTab>>,
    fn_id: Option<String>,
    count_optional_arg: usize
}
impl VarTab{
    fn new(id: Option<String>) -> VarTab {
        VarTab{v: Vec::new(),
            count_local: 0, count_arg: 0, count_context: 0, count_global: 0,
            context: None, fn_id: id, count_optional_arg: 0
        }
    }
    fn index_type(&mut self, id: &str) -> Option<(usize,VarType)> {
        if let Some(ref s) = self.fn_id {
            if id==s {return Some((0,VarType::FnId));}
        }
        { 
            let a = &self.v[..];
            for i in 0..a.len() {
                if a[i].s==id {
                    return Some((a[i].index, a[i].var_type));
                }
            }
        }
        if let Some(ref mut context) = self.context {
            if let Some((_,var_type)) = context.index_type(id) {
                if var_type == VarType::Global {
                    return None;
                }
                self.v.push(VarInfo{index: self.count_context,
                    s: id.to_string(), var_type: VarType::Context
                });
                self.count_context+=1;
                return Some((self.count_context-1,VarType::Context));
            }else{
                return None;
            }
        }else{
            return None;
        }
    }
}

pub struct JmpInfo{
    start: usize,
    breaks: Vec<usize>
}

pub struct Pool{
    stab: HashMap<String,usize>,
    stab_index: usize,
    data: Vec<Object>
}

impl Pool{
    fn get_index(&mut self, key: &str) -> usize {
        if let Some(index) = self.stab.get(key) {
            return *index;
        }
        self.data.push(CharString::new_object_str(key));
        self.stab.insert(String::from(key),self.stab_index);
        self.stab_index+=1;
        return self.stab_index-1;
    }
}

pub struct Compilation<'a>{
    mode_cmd: bool,
    syntax_nesting: usize,
    parens: usize,
    statement: bool,
    history: &'a mut system::History,
    file: &'a str,
    pool: Pool,
    bv_blocks: Vec<u32>,
    fn_indices: Vec<usize>,
    vtab: VarTab,
    function_nesting: usize,
    jmp_stack: Vec<JmpInfo>,
    coroutine: bool,
    for_nesting: usize,
    debug_mode: bool
}

struct TokenIterator{
    pub a: Rc<Box<[Token]>>,
    pub index: usize
}

impl TokenIterator{
    fn next_any_token(&mut self, c: &mut Compilation) -> Result<Rc<Box<[Token]>>,Error>{
        loop{
            if c.syntax_nesting>0 && c.mode_cmd {
                let value = self.a[self.index].value;
                if value == Symbol::Terminal {
                    let line = self.a[self.index].line;
                    let v = scan_line(line+1,c.history,true)?;
                    self.a = Rc::new(v.into_boxed_slice());
                    self.index = 0;
                }else{
                    return Ok(self.a.clone());
                }
            }else{
                return Ok(self.a.clone());
            }
        }
    }
    fn next_token(&mut self, c: &mut Compilation) -> Result<Rc<Box<[Token]>>,Error>{
        loop{
            let value = self.a[self.index].value;
            if c.mode_cmd && value == Symbol::Terminal {
                let line = self.a[self.index].line;
                let v = scan_line(line+1,c.history,false)?;
                self.a = Rc::new(v.into_boxed_slice());
                self.index = 0;
            }else if value == Symbol::Newline {
                self.index+=1;
            }else{
                return Ok(self.a.clone());
            }
        }
    }
    fn next_token_optional(&mut self, c: &mut Compilation) -> Result<Rc<Box<[Token]>>,Error>{
        loop{
            let value = self.a[self.index].value;
            if c.syntax_nesting>0 && c.mode_cmd && value == Symbol::Terminal {
                let line = self.a[self.index].line;
                let v = scan_line(line+1,c.history,true)?;
                self.a = Rc::new(v.into_boxed_slice());
                self.index = 0;
            }else if c.parens>0 && value == Symbol::Newline {
                self.index+=1;
            }else{
                return Ok(self.a.clone());
            }
        }
    }
}

fn operator(value: Symbol, a: Box<[Rc<AST>]>, line: usize, col: usize)
-> Rc<AST>
{
    Rc::new(AST{line, col, symbol_type: SymbolType::Operator,
        value, info: Info::None, s: None, a: Some(a)})
}

fn unary_operator(line: usize, col: usize,
    value: Symbol, x: Rc<AST>) -> Rc<AST>
{
    Rc::new(AST{line: line, col: col, symbol_type: SymbolType::Operator,
        value: value, info: Info::None, s: None, a: Some(Box::new([x]))})
}

fn binary_operator(line: usize, col: usize, value: Symbol,
    x: Rc<AST>, y: Rc<AST>) -> Rc<AST>
{
    Rc::new(AST{line: line, col: col, symbol_type: SymbolType::Operator,
        value: value, info: Info::None, s: None, a: Some(Box::new([x,y]))})
}

fn assignment(line: usize, col: usize, x: Rc<AST>, y: Rc<AST>) -> Rc<AST> {
    binary_operator(line,col,Symbol::Assignment,x,y)
}

fn atomic_literal(line: usize, col: usize, value: Symbol) -> Rc<AST>{
    Rc::new(AST{line: line, col: col, symbol_type: SymbolType::Keyword,
        value: value, info: Info::None, s: None, a: None})
}

fn symbol_none(line: usize, col: usize) -> Rc<AST>{
    Rc::new(AST{line: line, col: col, symbol_type: SymbolType::None,
        value: Symbol::None, info: Info::None, s: None, a: None})
}

fn identifier(id: &str, line: usize, col: usize) -> Rc<AST>{
    Rc::new(AST{line: line, col: col, symbol_type: SymbolType::Identifier,
        value: Symbol::None, info: Info::None, s: Some(id.to_string()), a: None})
}

fn string(id: String, line: usize, col: usize) -> Rc<AST>{
    Rc::new(AST{line: line, col: col, symbol_type: SymbolType::String,
        value: Symbol::None, info: Info::None, s: Some(id), a: None})
}

fn apply(line: usize, col: usize, a: Box<[Rc<AST>]>) -> Rc<AST> {
    Rc::new(AST{line: line, col: col, symbol_type: SymbolType::Operator,
        value: Symbol::Application, info: Info::None, s: None, a: Some(a)})
}

fn empty_list(line: usize, col: usize,
    symbol_type: SymbolType, symbol: Symbol
) -> Rc<AST>
{
    Rc::new(AST{line: line, col: col,
        symbol_type: symbol_type, value: symbol,
        info: Info::None, s: None, a: Some(Box::new([]))
    })
}

fn unary_node(line: usize, col: usize,
    symbol_type: SymbolType, value: Symbol, x: Rc<AST>
) -> Rc<AST>
{
    Rc::new(AST{line: line, col: col, symbol_type, value,
        info: Info::None, s: None, a: Some(Box::new([x]))})
}

fn binary_node(line: usize, col: usize,
    symbol_type: SymbolType, value: Symbol, x: Rc<AST>, y: Rc<AST>
) -> Rc<AST>
{
    Rc::new(AST{line: line, col: col, symbol_type, value,
        info: Info::None, s: None, a: Some(Box::new([x,y]))})
}

fn ast_node(line: usize, col: usize,
    symbol_type: SymbolType, value: Symbol, a: Box<[Rc<AST>]>
) -> Rc<AST>
{
    Rc::new(AST{line: line, col: col, symbol_type, value,
        info: Info::None, s: None, a: Some(a)})
}


impl<'a> Compilation<'a>{

#[inline(never)]
fn syntax_error(&self, line: usize, col: usize, s: &str) -> Error{
    Box::new(EnumError::Syntax(SyntaxError{
        line: line, col: col,
        file: String::from(self.file), s: String::from(s)
    }))
}

fn unexpected_token(&mut self, t: &Token) -> Error{
    let s = match t.token_type {
        SymbolType::Identifier => String::from("unexpected token: identifier."),
        SymbolType::String => String::from("unexpected token: string literal."),
        SymbolType::Int => String::from("unexpected token: integer literal."),
        _ => format!("unexpected token: '{}'.",symbol_to_string(t.value))
    };
    Box::new(EnumError::Syntax(SyntaxError{
        line: t.line, col: t.col, file: String::from(self.file),
        s: s
    }))
}

fn list_literal(&mut self, i: &mut TokenIterator) -> Result<Box<[Rc<AST>]>,Error> {
    let mut v: Vec<Rc<AST>> = Vec::new();
    let p = i.next_token(self)?;
    let t = &p[i.index];
    if t.value==Symbol::BRight {
        i.index+=1;
        return Ok(v.into_boxed_slice());
    }
    loop{
        let x = self.expression(i)?;
        v.push(x);
        let p = i.next_token(self)?;
        let t = &p[i.index];
        if t.value==Symbol::Comma {
            i.index+=1;
            let p = i.next_token(self)?;
            let t = &p[i.index];
            if t.value==Symbol::BRight {
                i.index+=1;
                break;
            }
        }else if t.value==Symbol::BRight {
            i.index+=1;
            break;
        }else{
            return Err(self.syntax_error(t.line, t.col, "expected ',' or ']'."));
        }
    }
    return Ok(v.into_boxed_slice());
}

fn map_literal(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error> {
    let mut v: Vec<Rc<AST>> = Vec::new();
    let p = i.next_token(self)?;
    let t = &p[i.index];
    if t.value == Symbol::CRight {
        i.index+=1;
    }else{
        'cycle: loop{
            'cright_after_comma: loop{
            'comma_or_cright: loop {
            let p = i.next_token(self)?;
            let t = &p[i.index];
            let key = if t.value == Symbol::Function {
                i.index+=1;
                let literal = self.function_literal(i,t,Symbol::Function)?;
                let a = ast_argv(&literal);

                let key = Rc::new(AST{
                    line: a[0].line, col: a[0].col,
                    symbol_type: SymbolType::String, value: Symbol::None,
                    info: Info::None, s: a[0].s.clone(), a: None
                });
                v.push(key);
                v.push(a[1].clone());
                break 'comma_or_cright;
            }else{
                self.union(i)?
            };
            let p = i.next_token(self)?;
            let t = &p[i.index];
            if t.value == Symbol::Comma {
                let value = symbol_none(t.line,t.col);
                v.push(key);
                v.push(value);
                i.index+=1;
                break 'cright_after_comma;
            }else if t.value == Symbol::CRight {
                let value = symbol_none(t.line,t.col);
                v.push(key);
                v.push(value);
                i.index+=1;
                break 'cycle;
            }else if t.value == Symbol::Colon {
                i.index+=1;
                let value = self.expression(i)?;
                v.push(key);
                v.push(value);
            }else if t.value== Symbol::Assignment {
                i.index+=1;
                if key.symbol_type != SymbolType::Identifier {
                    return Err(self.syntax_error(t.line, t.col, "expected an identifier before '='."));
                }
                let value = self.expression(i)?;
                let skey = Rc::new(AST{
                    line: key.line, col: key.col,
                    symbol_type: SymbolType::String, value: Symbol::None,
                    info: Info::None, s: key.s.clone(), a: None
                });
                v.push(skey);
                v.push(value);
            }else{
                return Err(self.syntax_error(t.line, t.col, "expected ',' or '=' or ':' or '}'."));
            }

            break;} // comma_or_cright:
            let p2 = i.next_token(self)?;
            let t2 = &p2[i.index];
            if t2.value == Symbol::CRight {
                i.index+=1;
                break 'cycle;
            }else if t2.value != Symbol::Comma {
                return Err(self.syntax_error(t2.line, t2.col, "expected ',' or '}'."));
            }
            i.index+=1;

            break;} // cright_after_comma:
            let p = i.next_token(self)?;
            let t = &p[i.index];
            if t.value == Symbol::CRight {
                i.index+=1;
                break;
            }
        }
    }
    return Ok(Rc::new(AST{line: t.line, col: t.col,
        symbol_type: SymbolType::Operator, value: Symbol::Map,
        info: Info::None, s: None, a: Some(v.into_boxed_slice())
    }));
}

fn table_literal(&mut self, i: &mut TokenIterator, t0: &Token) -> Result<Rc<AST>,Error> {
    let p = i.next_token(self)?;
    let t = &p[i.index];
    let prototype = if t.value==Symbol::CLeft {
        atomic_literal(t0.line, t0.col, Symbol::Null)
    }else{
        self.atom(i)?
    };
    let p = i.next_token(self)?;
    let t = &p[i.index];
    let map = if t.value == Symbol::CLeft {
        i.index+=1;
        self.map_literal(i)?
    }else if t.value == Symbol::PLeft {
        self.atom(i)?
    }else{
        return Err(self.syntax_error(t.line, t.col, "expected '{'."));
    };
    return Ok(Rc::new(AST{line: t0.line, col: t0.col, symbol_type: SymbolType::Keyword,
        value: Symbol::Table, info: Info::None, s: None, a: Some(Box::new([prototype,map]))}));
}

fn arguments_list(&mut self, i: &mut TokenIterator, t0: &Token, terminator: Symbol)
    -> Result<Rc<AST>,Error>
{
    let mut v: Vec<Rc<AST>> = Vec::new();
    let mut selfarg = false;
    let mut variadic = false;
    let p = i.next_token(self)?;
    let t = &p[i.index];
    if t.value == terminator {
        i.index+=1;
    }else{
        loop{
            let p = i.next_token(self)?;
            let t = &p[i.index];
            if t.value == Symbol::Ast {
                variadic = true;
                i.index+=1;
                let x = self.atom(i)?;
                v.push(x);
                let p = i.next_token(self)?;
                let t = &p[i.index];
                if t.value == terminator {
                    i.index+=1;
                    break;
                }else{
                    return Err(self.syntax_error(t.line, t.col, "expected '|'."));
                }
            }
            let x = self.atom(i)?;
            let p = i.next_token(self)?;
            let t = &p[i.index];
            if t.value == Symbol::Assignment {
                i.index+=1;
                let y = self.addition(i)?;
                v.push(binary_operator(t.line,t.col,Symbol::Assignment,x,y));
            }else{
                v.push(x);
            }
            let p = i.next_token(self)?;
            let t = &p[i.index];     
            if t.value == Symbol::Comma {
                i.index+=1;
                let p = i.next_token(self)?;
                let t = &p[i.index];
                if t.value==terminator {
                    i.index+=1;
                    break;
                }
            }else if t.value == Symbol::Semicolon {
                i.index+=1;
                selfarg = true;
                let p = i.next_token(self)?;
                let t = &p[i.index];
                if t.value == terminator {
                    i.index+=1;
                    break;
                }
            }else if t.value==terminator {
                i.index+=1;
                break;
            }else{
                return Err(self.syntax_error(t.line, t.col, "expected ',' or '|'."));
            }
        }
    }
    let info = if selfarg || variadic {
        Info::ArgvInfo{selfarg: selfarg, variadic: variadic}
    }else{
        Info::None
    };
    return Ok(Rc::new(AST{line: t0.line, col: t0.col,
        symbol_type: SymbolType::Operator, value: Symbol::List,
        info: info, s: None, a: Some(v.into_boxed_slice())
    }));
}

fn concise_function_literal(&mut self, i: &mut TokenIterator, t0: &Token)
    -> Result<Rc<AST>,Error>
{
    let args = self.arguments_list(i,t0,Symbol::Vline)?;
    let x = self.expression(i)?;
    return Ok(Rc::new(AST{line: t0.line, col: t0.col,
        symbol_type: SymbolType::Keyword, value: Symbol::Fn,
        info: Info::None, s: None, a: Some(Box::new([args,x]))
    }));
}

fn function_literal(&mut self, i: &mut TokenIterator,
    t0: &Token, symbol: Symbol
) -> Result<Rc<AST>,Error>
{
    let p = i.next_token(self)?;
    let t = &p[i.index];
    let (info,coroutine) = if t.value == Symbol::Ast {
        i.index+=1;
        (Info::Coroutine,true)
    }else{
        (Info::None,false)
    };
    let p = i.next_token(self)?;
    let t = &p[i.index];
    let id = if t.token_type == SymbolType::Identifier {
        i.index+=1;
        Some(t.item.assert_string().clone())
    }else{
        None
    };
    let p = i.next_token_optional(self)?;
    let t = &p[i.index];
    let terminator = if symbol == Symbol::Function {
        if t.value == Symbol::PLeft {
            i.index+=1;
            Symbol::PRight
        }else if t.value == Symbol::Newline {
            i.index+=1;
            Symbol::Newline
        }else if t.value == Symbol::Terminal {
            Symbol::Newline
        }else{
            return Err(self.syntax_error(t.line, t.col, "expected '(' or new line."));
        }
    }else{
        if t.value == Symbol::Vline {
            i.index+=1;
            Symbol::Vline
        }else{
            return Err(self.syntax_error(t.line, t.col, "expected '|'."));
        }
    };
    let args = if terminator == Symbol::Newline {
        operator(Symbol::List,Box::new([]),t0.line,t0.col)
    }else{
        self.arguments_list(i,t0,terminator)?
    };

    let statement = self.statement;
    self.statement = true;
    self.function_nesting+=1;
    self.syntax_nesting+=1;
    let parens = self.parens;
    self.parens = 0;
    let x = self.statements(i,if coroutine {Value::Empty} else {Value::Null})?;
    self.function_nesting-=1;
    self.statement = statement;

    let p = i.next_token_optional(self)?;
    let t = &p[i.index];
    if t.value != Symbol::End {
        return Err(self.syntax_error(t.line, t.col, "expected 'end'."));
    }
    self.parens = parens;
    self.syntax_nesting-=1;
    i.index+=1;
    self.end_of(i,Symbol::Fn)?;
    let y = Rc::new(AST{line: t0.line, col: t0.col,
        symbol_type: SymbolType::Keyword, value: Symbol::Fn,
        info: info, s: id, a: Some(Box::new([args,x]))
    });
    if symbol == Symbol::Function {
        let id = identifier(match y.s {Some(ref s) => s, None => unreachable!()},t.line,t.col);
        return Ok(binary_operator(t.line,t.col,Symbol::Assignment,id,y));
    }else{
        return Ok(y); 
    }
}


fn application(&mut self, i: &mut TokenIterator, f: Rc<AST>, terminal: Symbol)
    -> Result<Rc<AST>,Error>
{
    let mut v: Vec<Rc<AST>> = Vec::new();
    let mut v_named: Vec<Rc<AST>> = Vec::new();
    let mut self_argument = Info::None;
    let line = f.line;
    let col = f.col;
    v.push(f);
    let p = i.next_token(self)?;
    let t = &p[i.index];
    if t.value == terminal {
        i.index+=1;
    }else{
        loop{
            let x = self.expression(i)?;
            let p = i.next_token(self)?;
            let t = &p[i.index];
            if t.value == Symbol::Assignment {
                if x.symbol_type == SymbolType::Identifier {
                    i.index+=1;
                    let y = self.expression(i)?;
                    if let Some(ref s) = x.s {
                        v_named.push(string(s.clone(),t.line,t.col));
                        v_named.push(y);
                    }else{
                        unreachable!();
                    }
                }else{
                    return Err(self.syntax_error(t.line, t.col,
                        "expected identifer before '='."
                    ));
                }
            }else{
                v.push(x);
            }
            let p = i.next_token(self)?;
            let t = &p[i.index];
            if t.value == Symbol::Comma {
                i.index+=1;
            }else if t.value == Symbol::Semicolon {
                i.index+=1;
                self_argument = Info::SelfArg;
                let p2 = i.next_token(self)?;
                let t2 = &p2[i.index];
                if t2.value == terminal {
                    i.index+=1;
                    break;
                }
            }else if t.value == terminal {
                i.index+=1;
                break;
            }else{
                return Err(self.unexpected_token(&t));
            }
        }
    }
    if v_named.len()>0 {
        let m = Rc::new(AST{line: t.line, col: t.col,
            symbol_type: SymbolType::Operator, value: Symbol::Map,
            info: Info::None, s: None, a: Some(v_named.into_boxed_slice())
        });
        v.push(m);
    }
    let value = if terminal==Symbol::PRight {
        Symbol::Application
    }else{
        Symbol::Index
    };
    return Ok(Rc::new(AST{line: line, col: col, symbol_type: SymbolType::Operator,
        value: value, info: self_argument, s: None, a: Some(v.into_boxed_slice())}));
}

fn block(&mut self, i: &mut TokenIterator, t0: &Token) -> Result<Rc<AST>,Error> {
    let statement = self.statement;
    self.statement = true;
    self.function_nesting+=1;
    self.syntax_nesting+=1;
    let parens = self.parens;
    self.parens = 0;
    let x = self.statements(i,Value::Null)?;
    self.function_nesting-=1;
    self.statement = statement;

    let p = i.next_token_optional(self)?;
    let t = &p[i.index];
    if t.value != Symbol::End {
        return Err(self.syntax_error(t.line, t.col, "expected 'end'."));
    }
    self.parens = parens;
    self.syntax_nesting-=1;
    i.index+=1;
    let y = Rc::new(AST{line: t0.line, col: t0.col,
        symbol_type: SymbolType::Keyword, value: Symbol::Fn,
        info: Info::None, s: None, a: Some(Box::new([
            empty_list(t0.line,t0.col,SymbolType::Operator,Symbol::List),
        x]))
    });
    return Ok(apply(t0.line,t0.col,Box::new([y])));
}

fn atom(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error> {
    let p = i.next_token(self)?;
    let t = &p[i.index];
    let y;
    if t.token_type==SymbolType::Int {
        i.index+=1;
        y = match t.item {
            Item::Int(x) => Rc::new(AST{
                line: t.line, col: t.col, symbol_type: t.token_type,
                value: t.value, info: Info::Int(x),
                s: None, a: None
            }),
            Item::String(ref x) => Rc::new(AST{
                line: t.line, col: t.col, symbol_type: t.token_type,
                value: t.value, info: Info::Long,
                s: Some(x.clone()), a: None
            }),
            _ => unreachable!()
        };
    }else if t.token_type==SymbolType::Identifier ||
          t.token_type==SymbolType::String || t.token_type==SymbolType::Float ||
          t.token_type==SymbolType::Imag
    {
        i.index+=1;
        y = Rc::new(AST{line: t.line, col: t.col, symbol_type: t.token_type,
            value: t.value, info: Info::None,
            s: Some(t.item.assert_string().clone()),
            a: None});
    }else if t.value==Symbol::PLeft {
        i.index+=1;
        self.parens+=1;
        self.syntax_nesting+=1;
        y = self.comma_expression(i,Symbol::Tuple)?;
        let p = i.next_token(self)?;
        let t = &p[i.index];
        if t.value != Symbol::PRight {
            return Err(self.syntax_error(t.line, t.col, "expected ')'."));
        }
        self.syntax_nesting-=1;
        self.parens-=1;
        i.index+=1;
    }else if t.value==Symbol::BLeft {
        i.index+=1;
        self.parens+=1;
        self.syntax_nesting+=1;
        let x = self.list_literal(i)?;
        self.syntax_nesting-=1;
        self.parens-=1;
        y = Rc::new(AST{line: t.line, col: t.col,
            symbol_type: SymbolType::Operator, value: Symbol::List,
            info: Info::None, s: None, a: Some(x)
        });
    }else if t.value==Symbol::CLeft {
        i.index+=1;
        self.parens+=1;
        self.syntax_nesting+=1;
        y = self.map_literal(i)?;
        self.syntax_nesting-=1;
        self.parens-=1;
    }else if t.value==Symbol::Null ||
        t.value==Symbol::False || t.value==Symbol::True
    {
        i.index+=1;
        y = atomic_literal(t.line, t.col, t.value);
    }else if t.value==Symbol::Table {
        i.index+=1;
        y = self.table_literal(i,t)?;
    }else if t.value==Symbol::Vline {
        i.index+=1;
        y = self.concise_function_literal(i,t)?;
    }else if t.value==Symbol::Fn {
        i.index+=1;
        y = self.function_literal(i,t,Symbol::Fn)?;
    }else if t.value==Symbol::Begin {
        i.index+=1;
        y = self.block(i,t)?;
    }else{
        return Err(self.unexpected_token(&t));
    }
    return Ok(y);
}

fn application_term(&mut self, i: &mut TokenIterator)
    -> Result<Rc<AST>,Error>
{
    let mut x = self.atom(i)?;
    loop{
        let p = i.next_token_optional(self)?;
        let t = &p[i.index];
        if t.value == Symbol::PLeft {
            i.index+=1;
            self.parens+=1;
            self.syntax_nesting+=1;
            x = self.application(i,x,Symbol::PRight)?;
            self.syntax_nesting-=1;
            self.parens-=1;
        }else if t.value == Symbol::BLeft {
            i.index+=1;
            self.parens+=1;
            self.syntax_nesting+=1;
            x = self.application(i,x,Symbol::BRight)?;
            self.syntax_nesting-=1;
            self.parens-=1;
        }else if t.value == Symbol::CLeft {
            i.index+=1;
            self.parens+=1;
            self.syntax_nesting+=1;
            let y = self.map_literal(i)?;
            self.syntax_nesting-=1;
            self.parens-=1;
            x = Rc::new(AST{
                line: t.line, col: t.col,
                symbol_type: SymbolType::Operator,
                value: Symbol::Application, info: Info::None,
                s: None, a: Some(Box::new([x,y]))
            });
        }else if t.value == Symbol::Dot {
            i.index+=1;
            let p2 = i.next_token(self)?;
            let t2 = &p2[i.index];
            let y = if t2.token_type == SymbolType::Identifier {
                i.index+=1;
                Rc::new(AST{line: t2.line, col: t2.col,
                    symbol_type: SymbolType::String,
                    value: Symbol::None, info: Info::None,
                    s: Some(t2.item.assert_string().clone()), a: None
                })
            }else{
                self.atom(i)?
            };
            x = binary_operator(t.line,t.col,Symbol::Dot,x,y);
        }else{
            return Ok(x);
        }
    }
}

fn power(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error>{
    let x = self.application_term(i)?;
    let p = i.next_token_optional(self)?;
    let t = &p[i.index];
    if t.value==Symbol::Pow {
        i.index+=1;
        let y = self.signed_expression(i)?;
        return Ok(binary_operator(t.line,t.col,Symbol::Pow,x,y));
    }else{
        return Ok(x);
    }
}

fn signed_expression(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error>{
    let p = i.next_token(self)?;
    let t = &p[i.index];
    if t.value==Symbol::Minus || t.value==Symbol::Tilde || t.value==Symbol::Ast
    {
        i.index+=1;
        let x = self.power(i)?;
        let value = match t.value {
            Symbol::Minus => Symbol::Neg,
            Symbol::Ast => Symbol::Splat,
            _ => Symbol::Tilde
        };
        return Ok(unary_operator(t.line,t.col,value,x));
    }else if t.value==Symbol::Plus {
        i.index+=1;
        return self.power(i);
    }else{
        return self.power(i);
    }
}

fn multiplication(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error>{
    let mut y = self.signed_expression(i)?;
    let p = i.next_token_optional(self)?;
    let t = &p[i.index];
    let value=t.value;
    if value==Symbol::Ast || value==Symbol::Div ||
          value==Symbol::Mod || value==Symbol::Idiv
    {
        i.index+=1;
        let x = self.signed_expression(i)?;
        y = binary_operator(t.line,t.col,value,y,x);
        loop{
            let p = i.next_token_optional(self)?;
            let t = &p[i.index];
            let value = t.value;
            if value != Symbol::Ast && value != Symbol::Div &&
                  value != Symbol::Mod && value != Symbol::Idiv
            {
                return Ok(y);
            }
            i.index+=1;
            let x = self.signed_expression(i)?;
            y = binary_operator(t.line,t.col,value,y,x);  
        }
    }else{
        return Ok(y);
    }
}

fn addition(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error>{
    let mut y = self.multiplication(i)?;
    let p = i.next_token_optional(self)?;
    let t = &p[i.index];
    let value = t.value;
    if value==Symbol::Plus || value==Symbol::Minus {
        i.index+=1;
        let x = self.multiplication(i)?;
        y = binary_operator(t.line,t.col,value,y,x);
        loop{
            let p = i.next_token_optional(self)?;
            let t = &p[i.index];
            let value = t.value;
            if value != Symbol::Plus && value != Symbol::Minus {
                return Ok(y);
            }
            i.index+=1;
            let x = self.multiplication(i)?;
            y = binary_operator(t.line,t.col,value,y,x);  
        }
    }else{
        return Ok(y);
    }
}

fn shift(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error>{
    let x = self.addition(i)?;
    let p = i.next_token_optional(self)?;
    let t = &p[i.index];
    if t.value==Symbol::Lshift || t.value==Symbol::Rshift {
        i.index+=1;
        let y = self.addition(i)?;
        return Ok(binary_operator(t.line,t.col,t.value,x,y));
    }else{
        return Ok(x);
    }
}

fn intersection(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error>{
    let mut y = self.shift(i)?;
    let p = i.next_token_optional(self)?;
    let t = &p[i.index];
    let value = t.value;
    if value==Symbol::Amp {
        i.index+=1;
        let x = self.shift(i)?;
        y = binary_operator(t.line,t.col,value,y,x);
        loop{
            let p = i.next_token_optional(self)?;
            let t = &p[i.index];
            let value=t.value;
            if value != Symbol::Amp {
                return Ok(y);
            }
            i.index+=1;
            let x = self.shift(i)?;
            y = binary_operator(t.line,t.col,value,y,x);  
        }
    }else{
        return Ok(y);
    }
}

fn union(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error>{
    let mut y = self.intersection(i)?;
    let p = i.next_token_optional(self)?;
    let t = &p[i.index];
    let value = t.value;
    if value==Symbol::Vline || value==Symbol::Svert {
        i.index+=1;
        let x = self.intersection(i)?;
        y = binary_operator(t.line,t.col,value,y,x);
        loop{
            let p = i.next_token_optional(self)?;
            let t = &p[i.index];
            let value = t.value;
            if value != Symbol::Vline && value != Symbol::Svert {
                return Ok(y);
            }
            i.index+=1;
            let x = self.intersection(i)?;
            y = binary_operator(t.line,t.col,value,y,x);  
        }
    }else{
        return Ok(y);
    }
}

fn range(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error>{
    let x = self.union(i)?;
    let p = i.next_token_optional(self)?;
    let t = &p[i.index];
    if t.value == Symbol::Range {
        i.index+=1;
        let pb = i.next_token_optional(self)?;
        let tb = &pb[i.index];
        let y = if tb.value == Symbol::PRight ||
                   tb.value == Symbol::BRight ||
                   tb.value == Symbol::Colon  ||
                   tb.value == Symbol::Comma
        {
            atomic_literal(t.line,t.col,Symbol::Null)
        }else{
            self.union(i)?
        };
        let p2 = i.next_token_optional(self)?;
        let t2 = &p2[i.index];
        if t2.value == Symbol::Colon {
            i.index+=1;
            let d = self.union(i)?;
            return Ok(operator(Symbol::Range,Box::new([x,y,d]),t.line,t.col));
        }else{
            let d = atomic_literal(t.line,t.col,Symbol::Null);
            return Ok(operator(Symbol::Range,Box::new([x,y,d]),t.line,t.col));
        }
    }else{
        return Ok(x);
    }
}

fn comparison(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error>{
    let x = self.range(i)?;
    let p = i.next_token_optional(self)?;
    let t = &p[i.index];
    let value = t.value;
    if value==Symbol::Lt || value==Symbol::Gt ||
       value==Symbol::Le || value==Symbol::Ge
    {
        i.index+=1;
        let y = self.range(i)?;
        return Ok(binary_operator(t.line,t.col,value,x,y));
    }else{
        return Ok(x);
    }
}

fn eq_expression(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error>{
    let x = self.comparison(i)?;
    let p = i.next_token_optional(self)?;
    let t = &p[i.index];
    let value = t.value;
    if value==Symbol::Eq || value==Symbol::Ne    ||
       value==Symbol::Is || value==Symbol::Isin  ||
       value==Symbol::In || value==Symbol::Notin ||
       value==Symbol::Colon
    {
        i.index+=1;
        let y = self.comparison(i)?;
        return Ok(binary_operator(t.line,t.col,value,x,y));
    }else{
        return Ok(x);
    }
}

fn negation(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error>{
    let p = i.next_token(self)?;
    let t = &p[i.index];
    if t.value==Symbol::Not {
        i.index+=1;
        let x = self.eq_expression(i)?;
        return Ok(unary_operator(t.line,t.col,Symbol::Not,x));
    }else{
        return self.eq_expression(i);
    }
}

fn conjunction(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error>{
    let mut x = self.negation(i)?;
    let p = i.next_token_optional(self)?;
    let t = &p[i.index];
    if t.value==Symbol::And {
        i.index+=1;
        let y = self.negation(i)?;
        x = binary_operator(t.line,t.col,Symbol::And,x,y);
        loop{
            let p = i.next_token_optional(self)?;
            let t = &p[i.index];
            if t.value != Symbol::And {
                return Ok(x);
            }
            i.index+=1;
            let y = self.negation(i)?;
            x = binary_operator(t.line,t.col,Symbol::And,x,y);
        }
    }else{
        return Ok(x);
    }
}

fn disjunction(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error>{
    let mut x = self.conjunction(i)?;
    let p = i.next_token_optional(self)?;
    let t = &p[i.index];
    if t.value==Symbol::Or {
        i.index+=1;
        let y = self.conjunction(i)?;
        x = binary_operator(t.line,t.col,Symbol::Or,x,y);
        loop{
            let p = i.next_token_optional(self)?;
            let t = &p[i.index];
            if t.value != Symbol::Or {
                return Ok(x);
            }
            i.index+=1;
            let y = self.conjunction(i)?;
            x = binary_operator(t.line,t.col,Symbol::Or,x,y);
        }
    }else{
        return Ok(x);
    }
}

fn rec_for_if(&mut self, i: &mut TokenIterator, mut expr: Rc<AST>)
-> Result<Rc<AST>,Error>
{
    let p = i.next_any_token(self)?;
    let t = &p[i.index];
    if t.value == Symbol::For {
        i.index+=1;
        let mut variable = self.atom(i)?;
        let p = i.next_token(self)?;
        let t = &p[i.index];
        if t.value == Symbol::Comma {
            variable = self.comma_list(i,t,variable)?;
        }
        let p = i.next_token(self)?;
        let t = &p[i.index];
        if t.value != Symbol::In {
            return Err(self.syntax_error(t.line,t.col,"expected 'in'."));
        }
        i.index+=1;
        let a = self.disjunction(i)?;
        expr = self.rec_for_if(i,expr)?;
        return Ok(ast_node(t.line,t.col,SymbolType::Keyword,
          Symbol::For, Box::new([variable,a,expr])))
    }else if t.value == Symbol::If {
        i.index+=1;
        let condition = self.disjunction(i)?;
        expr = self.rec_for_if(i,expr)?;
        return Ok(binary_node(t.line,t.col,
            SymbolType::Keyword,Symbol::If,condition,expr));
    }else{
        return Ok(expr);
    }
}

fn for_expression(&mut self, i: &mut TokenIterator, mut expr: Rc<AST>)
-> Result<Rc<AST>,Error>
{
    expr = unary_node(expr.line,expr.col,
        SymbolType::Keyword,Symbol::Yield,expr);
    expr = self.rec_for_if(i,expr)?;
    let args = operator(Symbol::List,Box::new([]),expr.line,expr.col);
    let empty = atomic_literal(expr.line,expr.col,Symbol::Empty);
    expr = binary_node(expr.line, expr.col,
        SymbolType::Keyword, Symbol::Block, expr, empty);
    expr = Rc::new(AST{line: expr.line, col: expr.col,
        symbol_type: SymbolType::Keyword, value: Symbol::Fn,
        info: Info::Coroutine, s: None, a: Some(Box::new([args,expr]))});
    return Ok(expr);
}

fn if_expression(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error> {
    let x = self.disjunction(i)?;
    let p = i.next_token_optional(self)?;
    let t = &p[i.index];
    if t.value == Symbol::If {
        i.index+=1;
        let condition = self.disjunction(i)?;
        let p2 = i.next_token_optional(self)?;
        let t2 = &p2[i.index];
        if t2.value == Symbol::Else {
            i.index+=1;
            let y = self.expression(i)?;
            return Ok(Rc::new(AST{
                line: t.line, col: t.col, symbol_type: SymbolType::Operator,
                value: Symbol::If, info: Info::None,
                s: None, a: Some(Box::new([condition,x,y]))
            }));
        }else{
            return Ok(binary_operator(t.line,t.col,Symbol::If,condition,x));
        }
    }else if t.value == Symbol::Else {
        i.index+=1;
        let y = self.disjunction(i)?;
        return Ok(binary_operator(t.line,t.col,Symbol::Else,x,y));
    }else if t.value == Symbol::For {
        return self.for_expression(i,x);
    }else{
        return Ok(x);
    }
}

fn expression(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error>{
    return self.if_expression(i);
}

fn comma_expression(&mut self, i: &mut TokenIterator, value: Symbol) -> Result<Rc<AST>,Error>{
    let x = self.expression(i)?;
    let p = i.next_token_optional(self)?;
    let t = &p[i.index];
    if t.value == Symbol::Comma {
        let mut v: Vec<Rc<AST>> = Vec::new();
        v.push(x);
        i.index+=1;
        loop{
            let x = self.expression(i)?;
            v.push(x);
            let p = i.next_token_optional(self)?;
            let t = &p[i.index];
            if t.value == Symbol::Comma {
                i.index+=1;
            }else{
                break;
            }
        }
        return Ok(Rc::new(AST{line: t.line, col: t.col, symbol_type: SymbolType::Operator,
            value: value, info: Info::None, s: None, a: Some(v.into_boxed_slice())}));
    }else{
        return Ok(x);
    }
}

fn assignment(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error>{
    let x = self.comma_expression(i,Symbol::List)?;
    let p = i.next_token_optional(self)?;
    let t = &p[i.index];
    if t.value==Symbol::Assignment {
        i.index+=1;
        let y = self.comma_expression(i,Symbol::List)?;
        return Ok(binary_operator(t.line,t.col,Symbol::Assignment,x,y));
    }else if t.token_type == SymbolType::Assignment {
        i.index+=1;
        let y = self.expression(i)?;
        return Ok(Rc::new(AST{line: t.line, col: t.col, symbol_type: SymbolType::Assignment,
            value: t.value, info: Info::None, s: None, a: Some(Box::new([x,y]))}));
    }else if self.statement {
        return Ok(Rc::new(AST{line: t.line, col: t.col, symbol_type: SymbolType::Keyword,
            value: Symbol::Statement, info: Info::None, s: None, a: Some(Box::new([x]))}));
    }else{
        return Ok(x);
    }
}

fn while_statement(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error>{
    let condition = self.expression(i)?;
    let p = i.next_any_token(self)?;
    let t = &p[i.index];
    if t.value == Symbol::Do || t.value == Symbol::Newline {
        i.index+=1;
    }else{
        return Err(self.syntax_error(t.line, t.col, "expected 'do' or a line break."));
    }
    let body = self.statements(i,Value::None)?;
    return Ok(Rc::new(AST{line: t.line, col: t.col, symbol_type: SymbolType::Keyword,
        value: Symbol::While, info: Info::None, s: None, a: Some(Box::new([condition,body]))}));
}

fn comma_list(&mut self, i: &mut TokenIterator, t0: &Token, first: Rc<AST>) -> Result<Rc<AST>,Error> {
    i.index+=1;
    let mut v: Vec<Rc<AST>> = Vec::new();
    v.push(first);
    loop{
        let x = self.atom(i)?;
        v.push(x);
        let p = i.next_token(self)?;
        let t = &p[i.index];
        if t.value == Symbol::Comma {
            i.index+=1;
        }else{
            break;
        }
    }
    return Ok(Rc::new(AST{line: t0.line, col: t0.col, symbol_type: SymbolType::Operator,
        value: Symbol::List, info: Info::None, s: None, a: Some(v.into_boxed_slice())}));
}

fn for_statement(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error> {
    let mut variable = self.atom(i)?;
    let p = i.next_token(self)?;
    let t = &p[i.index];
    if t.value == Symbol::Comma {
        variable = self.comma_list(i,t,variable)?;
    }
    let p = i.next_token(self)?;
    let t = &p[i.index];
    if t.value != Symbol::In {
        return Err(self.syntax_error(t.line, t.col, "expected 'in'."));
    }
    i.index+=1;
    let a = self.comma_expression(i,Symbol::List)?;
    let p = i.next_any_token(self)?;
    let t = &p[i.index];
    if t.value == Symbol::Do || t.value == Symbol::Newline {
        i.index+=1;
    }else{
        return Err(self.syntax_error(t.line, t.col, "expected 'do' or a line break."));
    }
    let body = self.statements(i,Value::None)?;
    return Ok(Rc::new(AST{line: t.line, col: t.col, symbol_type: SymbolType::Keyword,
        value: Symbol::For, info: Info::None, s: None, a: Some(Box::new([variable,a,body]))}));
}

fn if_statement(&mut self, i: &mut TokenIterator, t0: &Token) -> Result<Rc<AST>,Error>{
    let mut v: Vec<Rc<AST>> = Vec::new();
    let condition = self.expression(i)?;
    let p = i.next_any_token(self)?;
    let t = &p[i.index];
    if t.value == Symbol::Then || t.value == Symbol::Newline{
        i.index+=1;
    }else{
        return Err(self.syntax_error(t.line, t.col,
            "expected 'then' or a line break."
        ));
    }
    let body = self.statements(i,Value::None)?;
    v.push(condition);
    v.push(body);
    loop{
        let p = i.next_token(self)?;
        let t = &p[i.index];
        if t.value == Symbol::Elif {
            i.index+=1;
            let condition = self.expression(i)?;
            let p = i.next_any_token(self)?;
            let t = &p[i.index];
            if t.value == Symbol::Then || t.value == Symbol::Newline {
                i.index+=1;
            }else{
                return Err(self.syntax_error(t.line, t.col, "expected 'then' or a line break."));
            }
            let body = self.statements(i,Value::None)?;
            v.push(condition);
            v.push(body);
        }else if t.value == Symbol::Else {
            i.index+=1;
            let body = self.statements(i,Value::None)?;
            v.push(body);
        }else if t.value == Symbol::End {
            break;
        }else{
            return Err(self.syntax_error(t.line,t.col,
                "expected 'elif, 'else' or 'end'."
            ));
        }
    }
    return Ok(Rc::new(AST{line: t0.line, col: t0.col, symbol_type: SymbolType::Keyword,
        value: Symbol::If, info: Info::None, s: None, a: Some(v.into_boxed_slice())}));
}

fn end_of(&mut self, i: &mut TokenIterator, symbol: Symbol) -> Result<(),Error>{
    let p = i.next_token_optional(self)?;
    let t = &p[i.index];
    if t.value == Symbol::Of {
        i.index+=1;
        let p = i.next_token_optional(self)?;
        let t = &p[i.index];
        if t.value != symbol {
            return Err(self.syntax_error(t.line, t.col,&format!(
                "expected 'end of {}', but got 'end of {}'.",
                symbol_to_string(symbol), symbol_to_string(t.value)
            )));
        }
        i.index+=1;
    }
    return Ok(());
}

fn try_catch_statement(
    &mut self, i: &mut TokenIterator, t0: &Token
) -> Result<Rc<AST>,Error>
{
    let mut v: Vec<Rc<AST>> = Vec::new();
    let block = self.statements(i,Value::None)?;
    v.push(block);
    let mut first = true;
    loop{
        let p = i.next_token(self)?;
        let t = &p[i.index];
        let cline;
        let ccol;
        if t.value == Symbol::Catch {
            i.index+=1;
            first = false;
            cline = t.line;
            ccol = t.col;
        }else if !first && t.value == Symbol::End {
            i.index+=1;
            break;
        }else{
            return Err(self.syntax_error(t.line, t.col, "expected 'catch'."));
        }
        let id = self.identifier(i)?;
        i.index+=1;
        let p = i.next_any_token(self)?;
        let t = &p[i.index];
        let expression = if t.value == Symbol::If {
            i.index+=1;
            Some(self.expression(i)?)
        }else if t.value == Symbol::Newline {
            i.index+=1;
            None
        }else{
            return Err(self.syntax_error(t.line, t.col, "expected 'if'."));   
        };
        let cblock = self.statements(i,Value::None)?;
        let c = Rc::new(AST{line: cline, col: ccol, symbol_type: SymbolType::Keyword,
            value: Symbol::Catch, info: Info::None, s: None,
            a: Some(if let Some(x)=expression {Box::new([id,x,cblock])}else{Box::new([id,cblock])})});
        v.push(c);
    }
    return Ok(Rc::new(AST{line: t0.line, col: t0.col, symbol_type: SymbolType::Keyword,
        value: Symbol::Try, info: Info::None, s: None, a: Some(v.into_boxed_slice())}));  
}

fn return_statement(&mut self, i: &mut TokenIterator, t0: &Token, symbol: Symbol
)
    -> Result<Rc<AST>,Error>
{
    let p = i.next_any_token(self)?;
    let t = &p[i.index];
    if t.value == Symbol::Newline {
        return Ok(Rc::new(AST{line: t0.line, col: t0.col, symbol_type: SymbolType::Keyword,
            value: symbol, info: Info::None, s: None, a: Some(Box::new([]))}));
    }else{
        let x = self.comma_expression(i,Symbol::List)?;
        return Ok(Rc::new(AST{line: t0.line, col: t0.col, symbol_type: SymbolType::Keyword,
            value: symbol, info: Info::None, s: None, a: Some(Box::new([x]))}));
    }
}

fn raise_statement(&mut self, i: &mut TokenIterator, t0: &Token) -> Result<Rc<AST>,Error>{
    let x = self.comma_expression(i,Symbol::List)?;
    return Ok(Rc::new(AST{line: t0.line, col: t0.col, symbol_type: SymbolType::Keyword,
        value: Symbol::Raise, info: Info::None, s: None, a: Some(Box::new([x]))}));
}

fn identifier(&mut self, i: &mut TokenIterator) -> Result<Rc<AST>,Error> {
    let p = i.next_token(self)?;
    let t = &p[i.index];
    let s = if t.token_type == SymbolType::Identifier {
        match t.item {Item::String(ref s) => s, _ => unreachable!()}
    }else{
        return Err(self.syntax_error(t.line, t.col, "expected an identifer."));
    };
    return Ok(identifier(s,t.line,t.col));
}

fn qualification_assignment(&mut self, id: &str, module: Rc<AST>,
    property: &str, line: usize, col: usize
) -> Rc<AST> {
    let id = identifier(id,line,col);
    let property = string(property.to_string(),line,col);
    let dot = binary_operator(line,col,Symbol::Dot,module,property);
    return assignment(line,col,id,dot);
}

fn qualification(&mut self, v: &mut Vec<Rc<AST>>, id: Rc<AST>,
    i: &mut TokenIterator, t0: &Token
) -> Result<(),Error>
{
    loop{
        let p = i.next_token(self)?;
        let t = &p[i.index];
        let s = if t.token_type == SymbolType::Identifier {
            match t.item {Item::String(ref s) => s, _ => unreachable!()}
        }else if t.value == Symbol::Ast {
            i.index+=1;
            // gtab().update(record(id))
            let line = t0.line;
            let col = t0.col;
            let gtab = identifier("gtab",line,col);
            let call_gtab = apply(line,col,Box::new([gtab]));
            let update = string("update".to_string(),line,col);
            let dot = binary_operator(line,col,Symbol::Dot,call_gtab,update);
            let record = identifier("record",line,col);
            let call_record_id = apply(line,col,Box::new([record,id]));
            let y = apply(line,col,Box::new([dot,call_record_id]));
            v.push(y);
            return Ok(());
        }else{
            return Err(self.syntax_error(t.line, t.col, "unexpected token."));
        };
        i.index+=1;
        let p2 = i.next_token_optional(self)?;
        let t2 = &p2[i.index];

        if t2.value == Symbol::Assignment {
            i.index+=1;
            let p2 = i.next_token(self)?;
            let t2 = &p2[i.index];
            if t2.token_type == SymbolType::Identifier {
                let s2 = match t2.item {Item::String(ref s) => s, _ => unreachable!()};
                let y = self.qualification_assignment(s,id.clone(),s2,t.line,t.col);
                v.push(y);
                i.index+=1;
                let p2 = i.next_token_optional(self)?;
                let t2 = &p2[i.index];
                if t2.value == Symbol::Comma {
                    i.index+=1;
                    continue;
                }else if t2.value == Symbol::Newline || t2.value == Symbol::Terminal ||
                    t2.value == Symbol::CRight
                {
                    break;
                }else{
                    return Err(self.syntax_error(t.line, t.col, "unexpected token."));
                }
            }else{
                return Err(self.syntax_error(t.line, t.col, "unexpected token."));
            }
        }else{
            let y = self.qualification_assignment(s,id.clone(),s,t.line,t.col);
            v.push(y);
        }
        if t2.value ==  Symbol::Comma {
            i.index+=1;
        }else if t2.value == Symbol::Newline || t2.value == Symbol::Terminal ||
            t2.value == Symbol::CRight
        {
            break;
        }else{
            return Err(self.syntax_error(t.line, t.col, "unexpected token."));
        }
    }
    return Ok(());
}

fn use_path(&mut self, i: &mut TokenIterator, t0: &Token)
    -> Result<(Rc<AST>,Option<Rc<AST>>),Error>
{
    let mut buffer = match t0.item {
        Item::String(ref s) => s.to_string(),
        _ => unreachable!()
    };
    buffer.push_str("/");
    i.index+=1;
    loop{
        let p = i.next_token(self)?;
        let t = &p[i.index];
        let s = if t.token_type == SymbolType::Identifier {
            match t.item {Item::String(ref s) => s, _ => unreachable!()}
        }else{
            return Err(self.syntax_error(t.line,t.col,"expected identifier."));
        };
        buffer.push_str(s);
        i.index+=1;
        let p2 = i.next_token_optional(self)?;
        let t2 = &p2[i.index];
        if t2.value == Symbol::Dot {
            buffer.push('/');
            i.index+=1;
        }else{
            let path = string(buffer,t0.line,t0.col);
            let id = identifier(s,t.line,t.col);
            return Ok((id,Some(path)));
        }
    }
}

fn use_statement(&mut self, i: &mut TokenIterator, t0: &Token)
    -> Result<Rc<AST>,Error>
{
    let mut v: Vec<Rc<AST>> = Vec::new();
    loop{
        let p = i.next_token(self)?;
        let t = &p[i.index];
        let (id,path) = if t.token_type == SymbolType::Identifier {
            let s = match t.item {Item::String(ref s) => s, _ => unreachable!()};
            let id = identifier(s,t.line,t.col);
            i.index+=1;
            let p2 = i.next_token_optional(self)?;
            let t2 = &p2[i.index];
            if t2.value == Symbol::Assignment {
                i.index+=1;
                let p3 = i.next_token(self)?;
                let t3 = &p3[i.index];
                if t3.token_type == SymbolType::Identifier {
                    i.index+=1;
                    let s2 = match t3.item {Item::String(ref s) => s, _ => unreachable!()};
                    let p4 = i.next_token_optional(self)?;
                    let t4 = &p4[i.index];
                    if t4.value == Symbol::Dot {
                        let (_,path) = self.use_path(i,t3)?;
                        (id,path)
                    }else{
                        let path = string(s2.to_string(),t.line,t.col);
                        (id,Some(path))
                    }
                }else{
                    return Err(self.syntax_error(t3.line, t3.col, "expected identifier."));
                }
            }else if t2.value == Symbol::Dot {
                self.use_path(i,t)?
            }else{
                let path = string(s.to_string(),t.line,t.col);
                (id,Some(path))
            }
        }else if t.value == Symbol::PLeft {
            i.index+=1;
            let id = self.identifier(i)?;
            i.index+=1;
            let p = i.next_token(self)?;
            let t = &p[i.index];
            if t.value == Symbol::PRight {
                i.index+=1;
            }else{
                return Err(self.syntax_error(t.line, t.col, "unexpected ')'."));
            }
            (id,None)
        }else{
            return Err(self.syntax_error(t.line, t.col, "unexpected identifier."));
        };

        if let Some(path) = path {
            let load = identifier("load",t.line,t.col);
            let app = apply(t.line,t.col,Box::new([load,path]));
            let y = assignment(t.line,t.col,id.clone(),app);
            v.push(y);
        }

        let p = i.next_token_optional(self)?;
        let t = &p[i.index];
        if t.value == Symbol::Comma {
            i.index+=1;
        }else if t.value == Symbol::Colon {
            i.index+=1;
            self.qualification(&mut v,id,i,t0)?;
            break;
        }else if t.value == Symbol::CLeft {
            i.index+=1;
            self.parens+=1;
            self.syntax_nesting+=1;
            self.qualification(&mut v,id,i,t0)?;
            self.parens-=1;
            self.syntax_nesting-=1;
            let p = i.next_token(self)?;
            let t = &p[i.index];
            if t.value == Symbol::CRight {
                i.index+=1;
            }else{
                return Err(self.syntax_error(t.line, t.col, "expected '}'."));
            }
            break;
        }else if t.value == Symbol::Newline || t.value == Symbol::Terminal {
            break;
        }else{
            return Err(self.syntax_error(t.line, t.col, "unexpected token."));
        }
    }
    if v.len()==1 {
        return Ok(match v.pop() {Some(x) => x, None => unreachable!()});
    }else{
        return Ok(Rc::new(AST{line: t0.line, col: t0.col, symbol_type: SymbolType::Keyword,
            value: Symbol::Block, info: Info::None, s: None, a: Some(v.into_boxed_slice())}));
    }
}

fn global_statement(&mut self, i: &mut TokenIterator, t0: &Token)
    -> Result<Rc<AST>,Error>
{
    let mut v: Vec<Rc<AST>> = Vec::new();
    loop{
        let x = self.atom(i)?;
        v.push(x);
        let p = i.next_token_optional(self)?;
        let t = &p[i.index];
        if t.value == Symbol::Comma {
            i.index+=1;
        }else{
            break;
        }
    }
    return Ok(Rc::new(AST{line: t0.line, col: t0.col, symbol_type: SymbolType::Keyword,
        value: Symbol::Global, info: Info::None, s: None, a: Some(v.into_boxed_slice())}));
}

fn assert_statement(&mut self, i: &mut TokenIterator, t0: &Token)
    -> Result<Rc<AST>,Error>
{
    let condition = self.expression(i)?;
    let p = i.next_any_token(self)?;
    let t = &p[i.index];
    if t.value == Symbol::Comma {
        i.index+=1;
        let text = self.expression(i)?;
        return Ok(binary_node(t0.line,t0.col,
            SymbolType::Keyword, Symbol::Assert, condition, text
        ));
    }else{
        return Ok(unary_node(t0.line,t0.col,
            SymbolType::Keyword, Symbol::Assert, condition
        ));
    }
}

fn statements(&mut self, i: &mut TokenIterator, last_value: Value)
    -> Result<Rc<AST>,Error>
{
    let mut v: Vec<Rc<AST>> = Vec::new();
    let p0 = i.next_token_optional(self)?;
    let t0 = &p0[i.index];
    loop{
        let p = i.next_any_token(self)?;
        let t = &p[i.index];
        if t.value == Symbol::Newline {
            i.index+=1;
            continue;
        }else if t.value == Symbol::Terminal {
            break;
        }
        let value = t.value;
        if t.token_type == SymbolType::Keyword {
            if value == Symbol::While {
                i.index+=1;
                let statement = self.statement;
                self.statement = true;
                self.syntax_nesting+=1;
                let x = self.while_statement(i)?;
                self.syntax_nesting-=1;
                self.statement = statement;
                v.push(x);
                let p = i.next_token_optional(self)?;
                let t = &p[i.index];
                if t.value != Symbol::End {    
                    return Err(self.syntax_error(t.line, t.col, "expected 'end'."));
                }
                i.index+=1;
                self.end_of(i,Symbol::While)?;
            }else if value == Symbol::For {
                i.index+=1;
                let statement = self.statement;
                self.statement = true;
                self.syntax_nesting+=1;
                let x = self.for_statement(i)?;
                self.syntax_nesting-=1;
                self.statement = statement;
                v.push(x);
                let p = i.next_token_optional(self)?;
                let t = &p[i.index];
                if t.value != Symbol::End {
                    return Err(self.syntax_error(t.line, t.col, "expected 'end'."));    
                }
                i.index+=1;
                self.end_of(i,Symbol::For)?;
            }else if value == Symbol::If {
                i.index+=1;
                let statement = self.statement;
                self.statement = true;
                self.syntax_nesting+=1;
                let x = self.if_statement(i,t)?;
                self.syntax_nesting-=1;
                self.statement = statement;
                v.push(x);
                let p = i.next_token_optional(self)?;
                let t = &p[i.index];
                if t.value != Symbol::End {
                    return Err(self.syntax_error(t.line, t.col, "expected 'end'."));
                }
                i.index+=1;
                self.end_of(i,Symbol::If)?;
            }else if value == Symbol::End || value == Symbol::Elif ||
                value == Symbol::Else || value == Symbol::Catch
            {
                break;
            }else if value == Symbol::Return {
                i.index+=1;
                let x = self.return_statement(i,t,Symbol::Return)?;
                v.push(x);
            }else if value == Symbol::Function {
                i.index+=1;
                let x = self.function_literal(i,t,Symbol::Function)?;
                v.push(x);
            }else if value == Symbol::Break {
                i.index+=1;
                let x = atomic_literal(t.line,t.col,Symbol::Break);
                v.push(x);
            }else if value == Symbol::Continue {
                i.index+=1;
                let x = atomic_literal(t.line,t.col,Symbol::Continue);
                v.push(x);
            }else if value == Symbol::Yield {
                i.index+=1;
                let x = self.return_statement(i,t,Symbol::Yield)?;
                v.push(x);
            }else if value == Symbol::Use {
                i.index+=1;
                let x = self.use_statement(i,t)?;
                v.push(x);
            }else if value == Symbol::Global {
                i.index+=1;
                let x = self.global_statement(i,t)?;
                v.push(x);
            }else if value == Symbol::Raise {
                i.index+=1;
                let x = self.raise_statement(i,t)?;
                v.push(x);
            }else if value == Symbol::Try {
                i.index+=1;
                let statement = self.statement;
                self.statement = true;
                self.syntax_nesting+=1;
                let x = self.try_catch_statement(i,t)?;
                self.syntax_nesting-=1;
                self.statement = statement;
                v.push(x);
            }else if value == Symbol::Assert {
                i.index+=1;
                let x = self.assert_statement(i,t)?;
                v.push(x);
            }else{
                let x = self.assignment(i)?;
                v.push(x);
            }
        }else{
            let x = self.assignment(i)?;
            v.push(x);
        }
        let p = i.next_any_token(self)?;
        let t = &p[i.index];
        let value = t.value;
        if value == Symbol::Semicolon {
            i.index+=1;
        }else if value == Symbol::End || value == Symbol::Elif ||
            value == Symbol::Else || value == Symbol::Catch
        {
            break;
        }else if value == Symbol::Newline {
            i.index+=1;
        }else if value == Symbol::Terminal {
            break;
        }else if value == Symbol::PRight {
            break;
        }else{
            return Err(self.unexpected_token(&t));
        }
    }
    if last_value != Value::None {
        let n = v.len();
        if n>0 && v[n-1].value == Symbol::Statement {
            if last_value == Value::Empty {
                v.push(empty_list(t0.line,t0.col,SymbolType::Keyword,Symbol::Yield));
                v.push(atomic_literal(t0.line,t0.col,Symbol::Empty));
            }else{
                let x = ast_argv(&v[n-1])[0].clone();
                v[n-1] = x;
            }
        }else if last_value == Value::Null {
            v.push(atomic_literal(t0.line,t0.col,Symbol::Null));
        }else if last_value == Value::Empty {
            v.push(atomic_literal(t0.line,t0.col,Symbol::Empty));
        }
    }
    if v.len()==1 {
        return Ok(match v.pop() {Some(x) => x, None => unreachable!()});
    }else{
        return Ok(Rc::new(AST{line: t0.line, col: t0.col, symbol_type: SymbolType::Keyword,
            value: Symbol::Block, info: Info::None, s: None, a: Some(v.into_boxed_slice())}));
    }
}

fn ast(&mut self, i: &mut TokenIterator, value: Value) -> Result<Rc<AST>,Error>{
    let y = self.statements(i,value)?;
    let p = i.next_any_token(self)?;
    let t = &p[i.index];
    if t.value == Symbol::Terminal {
        return Ok(y);
    }else{
        return Err(self.syntax_error(t.line, t.col, "unexpected token."));
    }
}

fn compile_operator(&mut self, bv: &mut Vec<u32>,
    t: &Rc<AST>, byte_code: u8
) -> Result<(),Error>
{
    let a = ast_argv(t);
    for i in 0..a.len() {
        self.compile_ast(bv,&a[i])?;
    }
    push_bc(bv,byte_code,t.line,t.col);
    return Ok(());
}

fn compile_map(&mut self, bv: &mut Vec<u32>,
  t: &Rc<AST>, byte_code: u8
) -> Result<(),Error>
{
    let a = ast_argv(t);
    for i in 0..a.len()/2 {
        self.compile_ast(bv,&a[2*i])?;
        let value = &a[2*i+1];
        if value.symbol_type == SymbolType::None {
            push_bc(bv,bc::NULL,value.line,value.col);
        }else{
            self.compile_ast(bv,value)?;
        }
    }
    push_bc(bv,byte_code,t.line,t.col);
    return Ok(());
}

// if   c1 then a1
// elif c2 then a2
// elif c3 then a3
// end
// c1 JPZ[1] a1 JMP[end] (1)
// c2 JPZ[2] a2 JMP[end] (2)
// c3 JPZ[3] a3 (3) (end)

// if c1 then a1
// elif c2 then a2
// elif c3 then a3
// else ae end
// c1 JPZ[1] a1 JMP[end] (1)
// c2 JPZ[2] a2 JMP[end] (2)
// c3 JPZ[3] a3 JMP[end] (3)
// ae (end)

fn compile_if(&mut self, v: &mut Vec<u32>, t: &Rc<AST>, is_op: bool)
    -> Result<(),Error>
{
    let a = ast_argv(t);
    let mut jumps: Vec<usize> = Vec::new();
    let m = a.len()/2;
    for i in 0..m {
        self.compile_ast(v,&a[2*i])?;
        push_bc(v, bc::JZ, a[2*i].line, a[2*i].col);
        let index = v.len();
        push_u32(v,0xcafe);
        self.compile_ast(v,&a[2*i+1])?;
        push_bc(v, bc::JMP, t.line, t.col);
        jumps.push(v.len());
        push_u32(v,0xcafe);
        let len = v.len();
        write_i32(&mut v[index..index+1],(BCSIZE+len) as i32-index as i32);
    }
    if a.len()%2==1 {
        self.compile_ast(v,&a[a.len()-1])?;
    }else{
        if is_op {
            push_bc(v, bc::NULL, t.line, t.col);
        }
    }
    let len = v.len();
    for i in 0..m {
        let index = jumps[i];
        write_i32(&mut v[index..index+1],(BCSIZE+len) as i32-index as i32);
    }
    return Ok(());
}


// while c do b end
// (1) c JPZ[2] b JMP[1] (2)

fn compile_while(&mut self, v: &mut Vec<u32>, t: &Rc<AST>)
    -> Result<(),Error>
{
    let index1 = v.len();
    let mut index2 = 0;
    self.jmp_stack.push(JmpInfo{start: index1, breaks: Vec::new()});
    let a = ast_argv(t);
    let condition = a[0].value != Symbol::True;
    
    if condition {
        self.compile_ast(v,&a[0])?;
        push_bc(v,bc::JZ,t.line,t.col);
        index2 = v.len();
        push_u32(v,0xcafe);
    }

    self.compile_ast(v,&a[1])?;
    push_bc(v,bc::JMP,t.line,t.col);
    let len = v.len();
    push_i32(v,(BCSIZE+index1) as i32-len as i32);

    if condition {
        let len = v.len();
        write_i32(&mut v[index2..index2+1],(BCSIZE+len) as i32-index2 as i32);
    }

    let info = match self.jmp_stack.pop() {
        Some(info) => info, None => unreachable!()
    };
    let len = v.len();
    for index in info.breaks {
        write_i32(&mut v[index..index+1],(BCSIZE+len) as i32-index as i32);
    }
    return Ok(());
}


// for x in a
//   print(x)
// end
//
// is translated into:
//
// _it_ = iter(a)
// while true
//   x = NEXT(_it_).BREAK_IF_EMPTY()
//   print(x)
// end
//
// These pseudo-functions NEXT and BREAK_IF_EMPTY are bundled
// into one byte code instruction called NEXT.

fn compile_for(&mut self, bv: &mut Vec<u32>, t: &Rc<AST>)
    -> Result<(),Error>
{
    let a = ast_argv(t);
    let iter = apply(t.line,t.col,Box::new([
        identifier("iter",t.line,t.col),
        a[1].clone()
    ]));
    let it = identifier(&format!("_it{}_",self.for_nesting),t.line,t.col);
    let assignment = binary_operator(t.line,t.col,Symbol::Assignment,it.clone(),iter);
    self.compile_ast(bv,&assignment)?;

    let start = bv.len();
    self.jmp_stack.push(JmpInfo{start: start, breaks: Vec::new()});

    self.compile_ast(bv,&it)?;
    push_bc(bv,bc::NEXT,it.line,it.col);
    let n = self.jmp_stack.len();
    self.jmp_stack[n-1].breaks.push(bv.len());
    push_i32(bv,0xcafe);
    self.compile_left_hand_side(bv,&a[0])?;

    self.compile_ast(bv,&a[2])?;
    push_bc(bv,bc::JMP,t.line,t.col);
    let len = bv.len();
    push_i32(bv,(BCSIZE+start) as i32-len as i32);

    let info = match self.jmp_stack.pop() {
        Some(info) => info, None => unreachable!()
    };
    let len = bv.len();
    for index in info.breaks {
        write_i32(&mut bv[index..index+1],(BCSIZE+len) as i32-index as i32);
    }

    return Ok(());
}

fn compile_try_catch(&mut self, bv: &mut Vec<u32>, t: &Rc<AST>)
    -> Result<(),Error>
{
    push_bc(bv,bc::OP,t.line,t.col);
    push_bc(bv,bc::TRY,t.line,t.col);
    let index1 = bv.len();
    push_i32(bv,0xcafe);

    let a = ast_argv(t);
    self.compile_ast(bv,&a[0])?;

    push_bc(bv,bc::OP,t.line,t.col);
    push_bc(bv,bc::TRYEND,t.line,t.col);
    push_bc(bv,bc::JMP,t.line,t.col);
    let index2 = bv.len();
    push_i32(bv,0xcafe);
    let len = bv.len();
    write_i32(&mut bv[index1..index1+1],(BCSIZE+len) as i32-index1 as i32);
    
    push_bc(bv,bc::OP,t.line,t.col);
    push_bc(bv,bc::GETEXC,t.line,t.col);
    let c = ast_argv(&a[1]);
    self.compile_assignment(bv,&c[0],c[0].line,c[0].col)?;
    if c.len()==2 {
        push_bc(bv,bc::OP,t.line,t.col);
        push_bc(bv,bc::TRYEND,t.line,t.col);
        self.compile_ast(bv,&c[1])?;
    }else{
        self.compile_ast(bv,&c[1])?;
        push_bc(bv,bc::JNZ,t.line,t.col);
        let index3 = bv.len();
        push_i32(bv,0xcafe);
        push_bc(bv,bc::OP,t.line,t.col);
        push_bc(bv,bc::CRAISE,t.line,t.col);
        let len = bv.len();
        write_i32(&mut bv[index3..index3+1],(BCSIZE+len) as i32-index3 as i32);   

        push_bc(bv,bc::OP,t.line,t.col);
        push_bc(bv,bc::TRYEND,t.line,t.col);
        self.compile_ast(bv,&c[2])?;
    }
    
    let len = bv.len();
    write_i32(&mut bv[index2..index2+1],(BCSIZE+len) as i32-index2 as i32);

    return Ok(());
}

fn compile_app_unpack(&mut self, bv: &mut Vec<u32>, a: &[Rc<AST>],
    self_argument: bool, line: usize, col: usize
) -> Result<(),Error>
{
    if self_argument {
        if a.len() != 3 {
            return Err(self.syntax_error(line,col,"expected one argument."));
        }
        self.compile_ast(bv,&a[0])?;
        self.compile_ast(bv,&a[1])?;
        let argv = &ast_argv(&a[2])[0];
        self.compile_ast(bv,argv)?;
    }else{
        if a.len() != 2 {
            return Err(self.syntax_error(line,col,"expected one argument."));
        }
        if a[0].value == Symbol::Dot {
            let b = ast_argv(&a[0]);
            self.compile_ast(bv,&b[0])?;
            self.compile_ast(bv,&b[1])?;
            push_bc(bv, bc::DUP_DOT_SWAP, line, col);
        }else{
            self.compile_ast(bv,&a[0])?;
            push_bc(bv, bc::NULL, line, col);
        }
        let argv = &ast_argv(&a[1])[0];
        self.compile_ast(bv,argv)?;
    }
    push_bc(bv, bc::APPLY, line, col);
    return Ok(());
}

fn compile_app(&mut self, v: &mut Vec<u32>, t: &Rc<AST>)
    -> Result<(),Error>
{
    let a = ast_argv(t);
    let self_argument = match t.info {
        Info::None => false,
        Info::SelfArg => true,
        _ => unreachable!()
    };
    let n = a.len();
    if a[n-1].value == Symbol::Splat {
        return self.compile_app_unpack(v,a,self_argument,t.line,t.col);
    }

    let argc = if self_argument {n-2} else {n-1};

    if self_argument {
        // callee
        self.compile_ast(v,&a[0])?;
    }else if a[0].value == Symbol::Dot{
        let b = ast_argv(&a[0]);
        self.compile_ast(v,&b[0])?;
        self.compile_ast(v,&b[1])?;
        push_bc(v, bc::DUP_DOT_SWAP, t.line, t.col);
    }else{
        // callee
        self.compile_ast(v,&a[0])?;

        // self argument
        push_bc(v, bc::NULL, t.line, t.col);
    }

    // arguments
    for i in 1..a.len() {
        self.compile_ast(v,&a[i])?;
    }

    push_bc(v, bc::CALL, t.line, t.col);

    // argument count,
    // not counting the self argument,
    // not counting the callee
    push_u32(v, argc as u32);

    return Ok(());
}

fn compile_fn(&mut self, bv: &mut Vec<u32>, t: &Rc<AST>)
    -> Result<(),Error>
{
    let a = ast_argv(t);
    let mut bv2: Vec<u32> = Vec::new();

    let (selfarg,variadic) = match a[0].info {
        Info::ArgvInfo{selfarg,variadic} => (selfarg,variadic),
        _ => (false,false)
    };

    // A separator to identify a new code block. Just needed
    // to make the assembler listing human readable.
    push_bc(&mut bv2, bc::FNSEP, t.line, t.col);

    // Move self.fn_indices beside to allow nested functions.
    let fn_indices = replace(&mut self.fn_indices,Vec::new());
    let jmp_stack = replace(&mut self.jmp_stack,Vec::new());

    // Every function has its own table of variables.
    let vtab = replace(&mut self.vtab,VarTab::new(t.s.clone()));
    self.vtab.context = Some(Box::new(vtab));

    self.function_nesting+=1;
    self.arguments(&mut bv2,&a[0],selfarg)?;
    let count_optional = self.vtab.count_optional_arg;

    let coroutine = self.coroutine;
    self.coroutine = match t.info {
        Info::Coroutine => true,
        _ => false
    };

    // Compile the function body.
    self.compile_ast(&mut bv2,&a[1])?;
    self.function_nesting-=1;
    
    let var_count = self.vtab.count_local;

    // Shift the start adresses of nested functions
    // by the now known offset and turn them into
    // position independent code. The offset is negative
    // because the code blocks of nested functions come
    // before this code block. So we need to jump back.
    self.offsets(&mut bv2,-(self.bv_blocks.len() as i32));

    // Restore self.fn_indices.
    replace(&mut self.fn_indices,fn_indices);
    self.jmp_stack = jmp_stack;

    // Add an additional return statement that will be reached
    // in case the control flow reaches the end of the function.
    push_bc(&mut bv2, bc::RET, t.line, t.col);
    
    // print_var_tab(&self.vtab,2);

    // Closure bindings.
    if self.vtab.count_context>0 {
        self.closure(bv,t);
    }else{
        push_bc(bv, bc::NULL, t.line, t.col);
    }

    // Restore.
    if let Some(context) = self.vtab.context.take() {
        self.vtab = *context;
    }
    self.coroutine = coroutine;

    // The name of the function.
    match t.s {
        Some(ref s) => {
            let index = self.pool.get_index(s);
            push_bc(bv,bc::STR,t.line,t.col);
            push_u32(bv,index as u32);
        },
        None => {
            push_bc(bv,bc::INT,t.line,t.col);
            push_u32(bv,((t.col as u32 & 0xffff)<<16) | (t.line as u32 & 0xffff));
            // let index = self.pool.get_index(&format!("({}:{})",t.line,t.col));
            // push_bc(bv,bc::STR,t.line,t.col);
            // push_u32(bv,index as u32);
            // push_bc(bv, bc::NULL, t.line, t.col);
        }
    }

    // Function constructor instruction.
    push_bc(bv, bc::FN, t.line, t.col);

    // Start address of the function body.
    // Add +1 to point behind FNSEP.
    // The size of bv will be added as the
    // compilation is finished.
    let index = bv.len();
    self.fn_indices.push(index);
    push_u32(bv,self.bv_blocks.len() as u32+1);

    let argv = ast_argv(&a[0]);

    let argc = if selfarg {
        argv.len()-1
    }else{
        argv.len()
    };

    if variadic {
        push_u32(bv,(argc-1) as u32);
        push_u32(bv,VARIADIC);
    }else{
        // minimal argument count
        push_u32(bv,(argc-count_optional) as u32);

        // maximal argument count
        push_u32(bv,argc as u32);
    }

    // number of local variables
    push_u32(bv,var_count as u32);

    // Append the code block to the buffer of code blocks.
    self.bv_blocks.append(&mut bv2);

    return Ok(());
}

fn offsets(&self, bv: &mut Vec<u32>, offset: i32){
    for &index in &self.fn_indices {
        let x = load_i32(&bv[index..index+1]);
        write_i32(&mut bv[index..index+1], x+BCSIZE as i32+offset-index as i32);
    }
}

fn string_literal(&mut self, s: &str, line: usize, col: usize)
    -> Result<String,Error>
{
    let mut y = String::new();
    let mut escape = false;
    let mut skip = false;
    let mut i = s.chars();
    loop{
        let c = match i.next() {Some(c) => c, None => break};
        if escape {
            if skip {
                if c==' ' || c=='\n' || c=='\t' {
                    continue;
                }else{
                    skip=false;
                    if c != '\\' {
                        escape=false;
                        y.push(c);
                    }
                    continue;
                }
            }
            if c=='n' {y.push('\n');}
            else if c=='s' {y.push(' ');}
            else if c=='t' {y.push('\t');}
            else if c=='d' {y.push('"');}
            else if c=='q' {y.push('\'');}
            else if c=='b' {y.push('\\');}
            else if c=='x' {
                let x = match unicode_literal(&mut i) {
                    Ok(x) => x, Err(e) => {
                        return Err(self.syntax_error(line,col,&e));
                    }
                };
                y.push(x);
            }
            else if c=='r' {y.push('\r');}
            else if c=='e' {y.push('\x1b');}
            else if c=='f' {y.push('\x0c');}
            else if c=='v' {y.push('\x0b');}
            else if c=='a' {y.push('\x07');}
            else if c==' ' || c=='\n' || c=='\t' {
                skip = true;
                continue;
            }
            else{y.push(c);}
            escape = false;
        }else if c == '\\' {
            escape = true;
        }else{
            y.push(c);
        }
    }
    return Ok(y);
}

fn compile_default_argument(&mut self, bv: &mut Vec<u32>, t: &Rc<AST>)
    -> Result<(),Error>
{
    let a = ast_argv(t);
    let null = atomic_literal(t.line,t.col,Symbol::Null);
    let condition = binary_operator(t.line,t.col,Symbol::Is,null,a[0].clone());
    let ast = Rc::new(AST{line: t.line, col: t.col, symbol_type: SymbolType::Keyword,
        value: Symbol::If, info: Info::None, s: None, a: Some(Box::new([condition,t.clone()]))
    });
    return self.compile_ast(bv,&ast);
}

fn arguments(&mut self, bv: &mut Vec<u32>, t: &AST, selfarg: bool)
    -> Result<(),Error>
{
    let a = ast_argv(t);

    if !selfarg {
        self.vtab.v.push(VarInfo{
            s: "self".to_string(),
            index: self.vtab.count_arg,
            var_type: VarType::Argument
        });
        self.vtab.count_arg+=1;
    }

    for i in 0..a.len() {
        if a[i].symbol_type == SymbolType::Identifier {
            if let Some(ref s) = a[i].s {
                self.vtab.v.push(VarInfo{
                    s: s.clone(),
                    index: self.vtab.count_arg,
                    var_type: VarType::Argument
                });
                self.vtab.count_arg+=1;
            }else{
                unreachable!();
            }
        }else if a[i].value == Symbol::List || a[i].value == Symbol::Map {
            let ids = format!("_t{}_",i);
            let helper = identifier(&ids,a[i].line,a[i].col);
            self.vtab.v.push(VarInfo{
                s: ids,
                index: self.vtab.count_arg,
                var_type: VarType::Argument
            });
            self.vtab.count_arg+=1;
            self.compile_ast(bv,&helper)?;
            self.compile_left_hand_side(bv,&a[i])?;
        }else if a[i].value == Symbol::Assignment {
            let u = ast_argv(&a[i]);
            if u[0].symbol_type == SymbolType::Identifier {
                if let Some(ref s) = u[0].s {
                    self.vtab.v.push(VarInfo{
                        s: s.clone(),
                        index: self.vtab.count_arg,
                        var_type: VarType::Argument
                    });
                    self.vtab.count_arg+=1;
                    self.vtab.count_optional_arg+=1;
                    self.compile_default_argument(bv,&a[i])?;
                }else{
                    unreachable!();
                }
            }else{
                return Err(self.syntax_error(a[i].line,a[i].col,
                    "expected identifier before '='."
                ));
            }
        }else{
            return Err(self.syntax_error(a[i].line,a[i].col,
                "expected identifier or [...] or {...} or assignment."
            ));
        }
    }
    return Ok(());
}

fn compile_variable(&mut self, bv: &mut Vec<u32>, t: &AST)
    -> Result<(),Error>
{
    let key = match t.s {Some(ref x) => x, None => unreachable!()};
    match self.vtab.index_type(key) {
        Some((index,var_type)) => {
            match var_type {
                VarType::Argument => {
                    push_bc(bv,bc::LOAD_ARG,t.line,t.col);
                    push_u32(bv,index as u32);
                },
                VarType::Local => {
                    push_bc(bv,bc::LOAD_LOCAL,t.line,t.col);
                    push_u32(bv,index as u32);
                },
                VarType::Context => {
                    push_bc(bv,bc::LOAD_CONTEXT,t.line,t.col);
                    push_u32(bv,index as u32);
                },
                VarType::Global => {
                    let index = self.pool.get_index(key);
                    push_bc(bv,bc::LOAD,t.line,t.col);
                    push_u32(bv,index as u32);    
                },
                VarType::FnId => {
                    push_bc(bv,bc::FNSELF,t.line,t.col);
                }
            }
            return Ok(());
        },
        None => {
            let index = self.pool.get_index(key);
            push_bc(bv,bc::LOAD,t.line,t.col);
            push_u32(bv,index as u32);
            return Ok(());
        }
    };
}

fn compile_assignment(&mut self, bv: &mut Vec<u32>, t: &AST,
    line: usize, col: usize
) -> Result<(),Error>
{
    let key = match t.s {Some(ref x) => x, None => unreachable!()};
    if self.function_nesting>0 {
        match self.vtab.index_type(key) {
            Some((index,var_type)) => {
                match var_type {
                    VarType::Local => {
                        push_bc(bv,bc::STORE_LOCAL,line,col);
                        push_u32(bv,index as u32);
                    },
                    VarType::Argument => {
                        push_bc(bv,bc::STORE_ARG,line,col);
                        push_u32(bv,index as u32);
                    },
                    VarType::Context => {
                        push_bc(bv,bc::STORE_CONTEXT,line,col);
                        push_u32(bv,index as u32);
                    },
                    VarType::Global => {
                        let index = self.pool.get_index(key);
                        push_bc(bv,bc::STORE,line,col);
                        push_u32(bv,index as u32);
                    },
                    VarType::FnId => {
                        return Err(self.syntax_error(t.line,t.col,
                            "cannot assign to function."
                        ));
                    }
                }
            },
            None => {
                if self.coroutine {
                    push_bc(bv,bc::STORE_CONTEXT,line,col);        
                    push_u32(bv,self.vtab.count_context as u32);
                    self.vtab.v.push(VarInfo{
                        s: key.clone(),
                        index: self.vtab.count_context,
                        var_type: VarType::Context
                    });
                    self.vtab.count_context+=1;
                }else{
                    push_bc(bv,bc::STORE_LOCAL,line,col);
                    push_u32(bv,self.vtab.count_local as u32);
                    self.vtab.v.push(VarInfo{
                        s: key.clone(),
                        index: self.vtab.count_local,
                        var_type: VarType::Local
                    });
                    self.vtab.count_local+=1;
                }
            }
        }
    }else{
        let index = self.pool.get_index(key);
        push_bc(bv,bc::STORE,line,col);
        push_u32(bv,index as u32);
    }
    return Ok(());
}

fn compile_compound_assignment(
    &mut self, bv: &mut Vec<u32>, t: &AST
) -> Result<(),Error> {
    let a = ast_argv(t);
    if a[0].symbol_type == SymbolType::Identifier {
        let value = match t.value {
            Symbol::APlus => Symbol::Plus,
            Symbol::AMinus => Symbol::Minus,
            Symbol::AAst => Symbol::Ast,
            Symbol::ADiv => Symbol::Div,
            Symbol::AIdiv => Symbol::Idiv,
            Symbol::AMod => Symbol::Mod,
            Symbol::AAmp => Symbol::Amp,
            Symbol::AVline => Symbol::Vline,
            Symbol::ASvert => Symbol::Svert,
            _ => unreachable!()
        };
        let op = binary_operator(t.line,t.col,value,a[0].clone(),a[1].clone());
        self.compile_ast(bv,&op)?;
        self.compile_assignment(bv,&a[0],t.line,t.col)?;
    }else{
        let args_left = ast_argv(&a[0]);
        self.compile_ast(bv,&args_left[0])?;
        self.compile_ast(bv,&args_left[1])?;
        self.compile_ast(bv,&a[1])?;

        let op_code = match t.value {
            Symbol::APlus => bc::ADD,
            Symbol::AMinus => bc::SUB,
            Symbol::AAst => bc::MPY,
            Symbol::ADiv => bc::DIV,
            Symbol::AIdiv => bc::IDIV,
            Symbol::AMod => bc::MOD,
            Symbol::AAmp => bc::BAND,
            Symbol::AVline => bc::BOR,
            _ => unreachable!()
        };

        push_bc(bv,bc::AOP,a[0].line,a[0].col);
        if a[0].value == Symbol::Index {
            push_bc(bv,bc::AOP_INDEX,a[0].line,a[0].col);
        }else if a[0].value == Symbol::Dot {
            push_bc(bv,bc::DOT,a[0].line,a[0].col);
        }else{
            return Err(self.syntax_error(a[0].line,a[0].col,
                "cannot compound assign to this expression."
            ));
        }
        push_bc(bv,op_code,a[0].line,a[0].col);
    }
    return Ok(());
}

fn closure(&mut self, bv: &mut Vec<u32>, t: &AST){
    let n = self.vtab.v.len();
    let a = &self.vtab.v[..];
    let ref mut context = match self.vtab.context {
        Some(ref mut context) => context,
        None => unreachable!()
    };
    for i in 0..n {
        if a[i].var_type == VarType::Context {
            if let Some((index,var_type)) = context.index_type(&a[i].s) {
                match var_type {
                    VarType::Local => {
                        push_bc(bv,bc::LOAD_LOCAL,t.line,t.col);
                        push_u32(bv,index as u32);
                    },
                    VarType::Argument => {
                        push_bc(bv,bc::LOAD_ARG,t.line,t.col);
                        push_u32(bv,index as u32);
                    },
                    VarType::Context => {
                        push_bc(bv,bc::LOAD_CONTEXT,t.line,t.col);
                        push_u32(bv,index as u32);
                    },
                    VarType::FnId => {
                        push_bc(bv,bc::FNSELF,t.line,t.col);
                    },
                    VarType::Global => {
                        // rare case:
                        // fn|| global k; fn*|| k=0 end end
                        let index = self.pool.get_index(&a[i].s);
                        push_bc(bv,bc::LOAD,t.line,t.col);
                        push_u32(bv,index as u32);
                    }
                }
            }else{
                if self.coroutine {
                    push_bc(bv,bc::NULL,t.line,t.col);
                }else{
                    println!("Error in closure: id '{}' not in context.",&a[i].s);
                    unreachable!();
                }
            }
        }
    }
    push_bc(bv,bc::LIST,t.line,t.col);
    push_u32(bv,self.vtab.count_context as u32);
}

fn global_declaration(&mut self, a: &[Rc<AST>]) -> Result<(),Error> {
    for t in a {
        if t.symbol_type == SymbolType::Identifier {
            let key = match t.s {Some(ref x) => x, None => unreachable!()};
            self.vtab.v.push(VarInfo{
                s: key.clone(),
                index: self.vtab.count_global,
                var_type: VarType::Global
            });
            self.vtab.count_global+=1;
        }else{
            return Err(self.syntax_error(t.line,t.col,
                "expected identifier."
            ));
        }
    }
    return Ok(());
}

fn compile_unpack(&mut self, bv: &mut Vec<u32>, t: &AST)
    -> Result<(),Error>
{
    let a = ast_argv(t);
    for i in 0..a.len() {
        push_bc(bv, bc::GET, a[i].line, a[i].col);
        push_u32(bv,i as u32);
        self.compile_left_hand_side(bv,&a[i])?;
    }
    push_bc(bv, bc::POP, t.line, t.col);
    return Ok(());
}

fn compile_left_hand_side(&mut self, bv: &mut Vec<u32>, t: &AST)
    -> Result<(),Error>
{
    if t.symbol_type == SymbolType::Identifier {
        self.compile_assignment(bv,&t,t.line,t.col)?;
    }else if t.value == Symbol::Index {
        let b = ast_argv(&t);
        for i in 0..b.len() {
            self.compile_ast(bv,&b[i])?;
        }
        push_bc(bv,bc::SET_INDEX,t.line,t.col);
        push_u32(bv,(b.len()-1) as u32);
    }else if t.value == Symbol::Dot {
        let b = ast_argv(&t);
        self.compile_ast(bv,&b[0])?;
        self.compile_ast(bv,&b[1])?;
        push_bc(bv,bc::DOT_SET,t.line,t.col);
    }else if t.value == Symbol::List {
        self.compile_unpack(bv,t)?;
    }else if t.value == Symbol::Map {
        let id = identifier("_m_",t.line,t.col);
        self.compile_assignment(bv,&id,t.line,t.col)?;
        let a = ast_argv(t);
        let n = a.len();
        let mut i=0;
        while i<n {
            if a[i+1].symbol_type == SymbolType::None {
                self.compile_ast(bv,&id)?;
                if a[i].symbol_type == SymbolType::Identifier {
                    self.compile_string(bv,&a[i])?;
                }else{
                    return Err(self.syntax_error(a[i].line,a[i].col,
                        "expected identifier."
                    ));
                }
                push_bc(bv,bc::GET_INDEX,t.line,t.col);
                push_u32(bv,1);
            }else{
                let app = apply(t.line,t.col,Box::new([id.clone(),a[i].clone()]));
                let null_coalescing = operator(
                    Symbol::Else, Box::new([app,a[i+1].clone()]),
                t.line,t.col);
                self.compile_ast(bv,&null_coalescing)?;
            }
            self.compile_assignment(bv,&a[i],t.line,t.col)?;
            i+=2;
        }
    }else{
        return Err(self.syntax_error(t.line,t.col,
            "expected identifier before '='."
        ));
    }
    return Ok(());
}

fn compile_string(&mut self, bv: &mut Vec<u32>, t: &AST)
    -> Result<(),Error>
{
    let s = match t.s {Some(ref x) => x, None => unreachable!()};
    let key = self.string_literal(s,t.line,t.col)?;
    let index = self.pool.get_index(&key);
    push_bc(bv,bc::STR,t.line,t.col);
    push_u32(bv,index as u32);
    return Ok(());
}

fn compile_long(&mut self, bv: &mut Vec<u32>, t: &AST)
    -> Result<(),Error>
{
    let s = match t.s {Some(ref x) => x, None => unreachable!()};
    let key = self.string_literal(s,t.line,t.col)?;
    let index = self.pool.get_index(&key);
    push_bc(bv,bc::LONG,t.line,t.col);
    push_u32(bv,index as u32);
    return Ok(());
}

fn compile_assert(&mut self, bv: &mut Vec<u32>, t: &AST)
    -> Result<(),Error>
{
    let a = ast_argv(t);
    let condition = unary_operator(t.line,t.col,Symbol::Not,a[0].clone());
    let err = if a.len()==2 {
        let x = string("Assertion failed: ".to_string(),t.line,t.col);
        let y = a[1].clone();
        binary_operator(t.line,t.col,Symbol::Plus,x,y)
    }else{
        string("Assertion failed.".to_string(),t.line,t.col)
    };
    let failed = unary_node(t.line,t.col,SymbolType::Keyword,Symbol::Raise,err);
    let y = binary_node(t.line,t.col,SymbolType::Keyword,Symbol::If,
        condition, failed
    );
    self.compile_ast(bv,&y)?;
    return Ok(());
}

fn compile_ast(&mut self, bv: &mut Vec<u32>, t: &Rc<AST>)
    -> Result<(),Error>
{
    if t.symbol_type == SymbolType::Identifier {
        self.compile_variable(bv,t)?;
    }else if t.symbol_type == SymbolType::Operator {
        let value = t.value;
        if value == Symbol::Assignment {
            let a = ast_argv(t);
            self.compile_ast(bv,&a[1])?;
            self.compile_left_hand_side(bv,&a[0])?;
        }else if value == Symbol::Plus {
            self.compile_operator(bv,t,bc::ADD)?;
        }else if value == Symbol::Minus {
            self.compile_operator(bv,t,bc::SUB)?;
        }else if value == Symbol::Ast {
            self.compile_operator(bv,t,bc::MPY)?;
        }else if value == Symbol::Div {
            self.compile_operator(bv,t,bc::DIV)?;
        }else if value == Symbol::Idiv {
            self.compile_operator(bv,t,bc::IDIV)?;
        }else if value == Symbol::Mod {
            self.compile_operator(bv,t,bc::MOD)?;
        }else if value == Symbol::Neg {
            self.compile_operator(bv,t,bc::NEG)?;
        }else if value == Symbol::Pow {
            self.compile_operator(bv,t,bc::POW)?;
        }else if value == Symbol::Amp {
            self.compile_operator(bv,t,bc::BAND)?;
        }else if value == Symbol::Vline {
            self.compile_operator(bv,t,bc::BOR)?;
        }else if value == Symbol::Svert {
            self.compile_operator(bv,t,bc::BXOR)?;
        }else if value == Symbol::Eq {
            self.compile_operator(bv,t,bc::EQ)?;
        }else if value == Symbol::Ne {
            self.compile_operator(bv,t,bc::NE)?;
        }else if value == Symbol::Lt {
            self.compile_operator(bv,t,bc::LT)?;
        }else if value == Symbol::Gt {
            self.compile_operator(bv,t,bc::GT)?;
        }else if value == Symbol::Le {
            self.compile_operator(bv,t,bc::LE)?;
        }else if value == Symbol::Ge {
            self.compile_operator(bv,t,bc::GE)?;
        }else if value == Symbol::Is {
            self.compile_operator(bv,t,bc::IS)?;
        }else if value == Symbol::Isnot {
            // self.compile_operator(bv,t,bc::ISNOT)?;
            self.compile_operator(bv,t,bc::IS)?;
            push_bc(bv,bc::NOT,t.line,t.col);
        }else if value == Symbol::In {
            self.compile_operator(bv,t,bc::IN)?;
        }else if value == Symbol::Notin {
            // self.compile_operator(bv,t,bc::NOTIN)?;
            self.compile_operator(bv,t,bc::IN)?;
            push_bc(bv,bc::NOT,t.line,t.col);
        }else if value == Symbol::Colon {
            self.compile_operator(bv,t,bc::OF)?;
        }else if value == Symbol::Index {
            self.compile_operator(bv,t,bc::GET_INDEX)?;
            let a = ast_argv(t);
            push_u32(bv,(a.len()-1) as u32);
        }else if value == Symbol::Not {
            self.compile_operator(bv,t,bc::NOT)?;
        }else if value == Symbol::Range {
            self.compile_operator(bv,t,bc::RANGE)?;
        }else if value == Symbol::List {
            self.compile_operator(bv,t,bc::LIST)?;
            let size = match t.a {Some(ref a) => a.len() as u32, None => unreachable!()};
            push_u32(bv,size);
        }else if value == Symbol::Map {
            self.compile_map(bv,t,bc::MAP)?;
            let size = match t.a {Some(ref a) => a.len() as u32, None => unreachable!()};
            push_u32(bv,size);
        }else if value == Symbol::Application {
            self.compile_app(bv,t)?;
        }else if value == Symbol::If {
            self.compile_if(bv,t,true)?;
        }else if value == Symbol::Dot {
            self.compile_operator(bv,t,bc::DOT)?;
        }else if value == Symbol::And {
            // We use a AND[1] b (1) instead of
            // a JPZ[1] b JMP[2] (1) CONST_BOOL false (2).
            let a = ast_argv(t);
            self.compile_ast(bv,&a[0])?;
            push_bc(bv,bc::AND,t.line,t.col);
            let index = bv.len();
            push_i32(bv,0xcafe);
            self.compile_ast(bv,&a[1])?;
            let len = bv.len();
            write_i32(&mut bv[index..index+1], (BCSIZE+len) as i32-index as i32);
        }else if value == Symbol::Or {
            // We use a OR[1] b (1) instead of
            // a JPZ[1] CONST_BOOL true JMP[2] (1) b (2).
            let a = ast_argv(t);
            self.compile_ast(bv,&a[0])?;
            push_bc(bv,bc::OR,t.line,t.col);
            let index = bv.len();
            push_i32(bv,0xcafe);
            self.compile_ast(bv,&a[1])?;
            let len = bv.len();
            write_i32(&mut bv[index..index+1], (BCSIZE+len) as i32-index as i32);
        }else if value == Symbol::Else {
            // a ELSE[1] b (1)
            let a = ast_argv(t);
            self.compile_ast(bv,&a[0])?;
            push_bc(bv,bc::ELSE,t.line,t.col);
            let index = bv.len();
            push_i32(bv,0xcafe);
            self.compile_ast(bv,&a[1])?;
            let len = bv.len();
            write_i32(&mut bv[index..index+1], (BCSIZE+len) as i32-index as i32);
        }else if value == Symbol::Tuple {
            self.compile_operator(bv,t,bc::TUPLE)?;
            let size = match t.a {Some(ref a) => a.len() as u32, None => unreachable!()};
            push_u32(bv,size);
        }else{
            return Err(self.syntax_error(t.line,t.col,&format!(
                "cannot compile Operator '{}'.", symbol_to_string(t.value)
            )));
        }
    }else if t.symbol_type == SymbolType::Int {
        match t.info {
            Info::Int(x) => {
                push_bc(bv, bc::INT, t.line, t.col);
                push_i32(bv,x);
            },
            Info::Long => {
                self.compile_long(bv,t)?;
            },
            _ => unreachable!()
        };
    }else if t.symbol_type == SymbolType::Float {
        push_bc(bv, bc::FLOAT, t.line, t.col);
        let x: f64 = match t.s {Some(ref x) => x.parse().unwrap(), None => unreachable!()};
        push_u64(bv,x.to_bits());
    }else if t.symbol_type == SymbolType::Imag {
        push_bc(bv, bc::IMAG, t.line, t.col);
        let x: f64 = match t.s {Some(ref x) => x.parse().unwrap(), None => unreachable!()};
        push_u64(bv,x.to_bits());
    }else if t.symbol_type == SymbolType::Keyword {
        let value = t.value;
        if value == Symbol::If {
            self.compile_if(bv,t,false)?;
        }else if value == Symbol::While {
            self.compile_while(bv,t)?;
        }else if value == Symbol::For {
            self.for_nesting+=1;
            self.compile_for(bv,t)?;
            self.for_nesting-=1;
        }else if value == Symbol::Block {
            let a = ast_argv(t);
            for i in 0..a.len() {
                self.compile_ast(bv,&a[i])?;
            }
        }else if value == Symbol::Fn {
            self.compile_fn(bv,t)?;
        }else if value == Symbol::Statement {
            let a = ast_argv(t);
            self.compile_ast(bv,&a[0])?;
            push_bc(bv,bc::POP,t.line,t.col);
        }else if value == Symbol::Return {
            let a = ast_argv(t);
            if a.len()==0 {
                push_bc(bv,bc::NULL,t.line,t.col);
            }else{
                self.compile_ast(bv,&a[0])?;
            }
            push_bc(bv,bc::RET,t.line,t.col);
        }else if value == Symbol::Raise {
            let a = ast_argv(t);
            self.compile_ast(bv,&a[0])?;
            push_bc(bv,bc::RAISE,t.line,t.col);
        }else if value == Symbol::Break {
            push_bc(bv,bc::JMP,t.line,t.col);
            // let index2 = v.len();
            let n = self.jmp_stack.len();
            if n==0 {
                return Err(self.syntax_error(t.line,t.col,
                    "Statement 'break' is expected to be inside of a loop."
                ));
            }
            let breaks = &mut self.jmp_stack[n-1].breaks;
            breaks.push(bv.len());
            push_u32(bv,0xcafe);
        }else if value == Symbol::Continue {
            push_bc(bv,bc::JMP,t.line,t.col);
            let start = match self.jmp_stack.last() {
                Some(info) => info.start,
                None => return Err(self.syntax_error(t.line,t.col,
                    "Statement 'continue' is expected to be inside of a loop."
                ))
            };
            let len = bv.len();
            push_i32(bv,(BCSIZE+start) as i32-len as i32);
        }else if value == Symbol::Null {
            push_bc(bv,bc::NULL,t.line,t.col);
        }else if value == Symbol::True {
            push_bc(bv,bc::TRUE,t.line,t.col);
        }else if value == Symbol::False {
            push_bc(bv,bc::FALSE,t.line,t.col);
        }else if value == Symbol::Empty {
            push_bc(bv,bc::EMPTY,t.line,t.col);
        }else if value == Symbol::Yield {
            if !self.coroutine {
                return Err(self.syntax_error(t.line,t.col,&format!(
                    "yield is only valid in fn*."
                )));
            }
            let a = ast_argv(t);
            if a.len()==0 {
                push_bc(bv,bc::NULL,t.line,t.col);
            }else{
                self.compile_ast(bv,&a[0])?;
            }
            push_bc(bv,bc::YIELD,t.line,t.col);
        }else if value == Symbol::Table {
            let a = ast_argv(t);
            self.compile_ast(bv,&a[1])?;
            self.compile_ast(bv,&a[0])?;
            push_bc(bv,bc::TABLE,t.line,t.col);
        }else if value == Symbol::Global {
            let a = ast_argv(t);
            self.global_declaration(a)?;
        }else if value == Symbol::Try {
            self.compile_try_catch(bv,t)?;
        }else if value == Symbol::Assert {
            if self.debug_mode {
                self.compile_assert(bv,t)?;
            }
        }else{
            unreachable!();
        }
    }else if t.symbol_type == SymbolType::String {
        self.compile_string(bv,t)?;
    }else if t.symbol_type == SymbolType::Assignment {
        self.compile_compound_assignment(bv,t)?;
    }else{
        unreachable!();
    }
    return Ok(());
}

}//impl Compilation

fn expect_char(i: &mut Chars, c: char) -> Result<char,String> {
    match i.next() {
        Some(y) => {
            if c=='*' || y==c {return Ok(y);} else {
                return Err(format!("after \\u: unexpected character: '{}'.",y));
            }
        },
        _ => return Err("in \\u{}: unexpected end of input.".to_string())
    }
}

fn unicode_literal(i: &mut Chars) -> Result<char,String> {
    expect_char(i,'{')?;
    let mut x: u32 = 0;
    loop{
        let c = expect_char(i,'*')?;
        if c=='}' {break;}
        else{
            x = 16*x+match c.to_digit(16) {
                Some(digit) => digit,
                None => return Err("in \\u{}: expected hexadecimal digits.".to_string())
            };
        }
    }
    return Ok(match char::from_u32(x) {Some(x) => x, None => '?'});
}

fn ast_argv(t: &AST) -> &Box<[Rc<AST>]>{
    match t.a {Some(ref x) => x, None => unreachable!()}
}

fn push_u32(bv: &mut Vec<u32>, x: u32){
    bv.push(x);
}

fn push_i32(bv: &mut Vec<u32>, x: i32){
    bv.push(x as u32);
}

fn write_i32(a: &mut [u32], x: i32){
    a[0] = x as u32;
}

fn push_u64(bv: &mut Vec<u32>, x: u64){
    bv.push(x as u32);
    bv.push((x>>32) as u32);
}



fn push_bc(bv: &mut Vec<u32>, byte: u8, line: usize, col: usize){
    bv.push(((col as u32)&0xff)<<24 | ((line as u32)&0xffff)<<8 | (byte as u32))
}

// fn compose_u16(b0: u8, b1: u8) -> u16 {
//     (b1 as u16)<<8 | (b0 as u16)
// }

fn load_i32(a: &[u32]) -> i32 {
    a[0] as i32
}

fn load_u32(a: &[u32]) -> u32 {
    a[0]
}

fn load_u64(a: &[u32]) -> u64 {
   (a[1] as u64)<<32 | (a[0] as u64)
}

fn asm_listing(a: &[u32]) -> String {
    let mut s = String::from("Adr | Line:Col| Operation\n");
    let mut i=0;
    while i<a.len() {
        let op = a[i] as u8;
        let line = ((a[i]>>8) & 0xffff) as u16;
        let col = (a[i]>>24) as u8;
        if op != bc::FNSEP {
            let u = format!("{:04}| {:4}:{:02} | ",i,line,col);
            s.push_str(&u);
        }
        match op {
            bc::INT => {
                let x = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("push int {} (0x{:x})\n",x,x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::FLOAT => {
                let x = f64::from_bits(
                    load_u64(&a[BCSIZE+i..BCSIZE+i+2])
                );
                let u = format!("push float {:e}\n",x);
                s.push_str(&u);
                i+=BCAASIZE;
            },
            bc::IMAG => {
                let x = f64::from_bits(
                    load_u64(&a[BCSIZE+i..BCSIZE+i+2])
                );
                let u = format!("push imag {:e}\n",x);
                s.push_str(&u);
                i+=BCAASIZE;
            },
            bc::NULL => {s.push_str("null\n"); i+=BCSIZE;},
            bc::TRUE => {s.push_str("true\n"); i+=BCSIZE;},
            bc::FALSE => {s.push_str("false\n"); i+=BCSIZE;},
            bc::FNSELF => {s.push_str("function self\n"); i+=BCSIZE;},
            bc::ADD => {s.push_str("add\n"); i+=BCSIZE;},
            bc::SUB => {s.push_str("sub\n"); i+=BCSIZE;},
            bc::MPY => {s.push_str("mpy\n"); i+=BCSIZE;},
            bc::DIV => {s.push_str("div\n"); i+=BCSIZE;},
            bc::IDIV => {s.push_str("idiv\n"); i+=BCSIZE;},
            bc::MOD => {s.push_str("mod\n"); i+=BCSIZE;},
            bc::POW => {s.push_str("pow\n"); i+=BCSIZE;},
            bc::NEG => {s.push_str("neg\n"); i+=BCSIZE;},
            bc::EQ => {s.push_str("eq\n"); i+=BCSIZE;},
            bc::NE => {s.push_str("not eq\n"); i+=BCSIZE;},
            bc::LT => {s.push_str("lt\n"); i+=BCSIZE;},
            bc::GT => {s.push_str("gt\n"); i+=BCSIZE;},
            bc::LE => {s.push_str("le\n"); i+=BCSIZE;},
            bc::GE => {s.push_str("not ge\n"); i+=BCSIZE;},
            bc::IS => {s.push_str("is\n"); i+=BCSIZE;},
            bc::OF => {s.push_str("of\n"); i+=BCSIZE;},
            bc::ISNOT => {s.push_str("is not\n"); i+=BCSIZE;},
            bc::IN => {s.push_str("in\n"); i+=BCSIZE;},
            bc::NOTIN => {s.push_str("not in\n"); i+=BCSIZE;},
            bc::NOT => {s.push_str("not\n"); i+=BCSIZE;},
            bc::RANGE => {s.push_str("range\n"); i+=BCSIZE;},
            bc::TABLE => {s.push_str("table\n"); i+=BCSIZE;},
            bc::LIST => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("list, size={}\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::MAP => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("map, size={}\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::LOAD => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("load global [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::LOAD_ARG => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("load argument [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::LOAD_LOCAL => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("load local [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::LOAD_CONTEXT => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("load context [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::STORE => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("store global [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::STORE_ARG => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("store argument [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::STORE_LOCAL => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("store local [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::STORE_CONTEXT => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("store context [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::STR => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("string literal [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::LONG => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("long literal [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::AND => {
                let x = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("and {}\n",i as i32+x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::OR => {
                let x = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("or {}\n",i as i32+x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::ELSE => {
                let x = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("else {}\n",i as i32+x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::JMP => {
                let x = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);

                // Resolve position independent code
                // to make the listing human readable.
                let u = format!("jmp {}\n",i as i32+x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::JZ => {
                let x = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("jz {}\n",i as i32+x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::JNZ => {
                let x = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("jnz {}\n",i as i32+x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::NEXT => {
                let x = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("next {}\n",i as i32+x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::GET => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("get {}\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::CALL => {
                let argc = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("call, argc={}\n",argc);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::RET => {s.push_str("ret\n"); i+=BCSIZE;},
            bc::YIELD => {s.push_str("yield\n"); i+=BCSIZE;},
            bc::RAISE => {s.push_str("raise\n"); i+=BCSIZE;},
            bc::FNSEP => {s.push_str("\nFunction\n"); i+=BCSIZE;},
            bc::FN => {
                let address = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);
                let argc_min = load_i32(&a[BCSIZE+i+1..BCSIZE+i+2]);
                let argc_max = load_i32(&a[BCSIZE+i+2..BCSIZE+i+3]);
                // let var_count = load_i32(&a[BCSIZE+i+3..BCSIZE+i+4]);

                // Resolve position independent code
                // to make the listing human readable.
                let u = format!("fn [{}], argc_min={}, argc_max={}\n",
                    i as i32+address, argc_min, argc_max
                );
                s.push_str(&u);
                i+=BCSIZE+4;
            },
            bc::GET_INDEX => {
                let argc = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                if argc>1 {
                    s.push_str(&format!("get index ({} args)\n",argc));
                }else{
                    s.push_str("get index\n");
                }
                i+=BCASIZE;
            },
            bc::SET_INDEX => {
                let argc = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                if argc>1 {
                    s.push_str(&format!("set index ({} args)\n",argc));
                }else{
                    s.push_str("set index\n");
                }
                i+=BCASIZE;
            },
            bc::DOT => {s.push_str("dot\n"); i+=BCSIZE;},
            bc::DOT_SET => {s.push_str("dot set\n"); i+=BCSIZE;},
            bc::SWAP => {s.push_str("swap\n"); i+=BCSIZE;},
            bc::DUP => {s.push_str("dup\n"); i+=BCSIZE;},
            bc::DUP_DOT_SWAP => {s.push_str("dup dot swap\n"); i+=BCSIZE;},
            bc::POP => {s.push_str("pop\n"); i+=BCSIZE;},
            bc::EMPTY => {s.push_str("empty\n"); i+=BCSIZE;},
            bc::AOP => {s.push_str("aop\n"); i+=BCSIZE;},
            bc::AOP_INDEX => {s.push_str("aop index\n"); i+=BCSIZE;},
            bc::OP => {
                s.push_str("op ");
                i+=BCSIZE;
                let op = a[i] as u8;
                if op==bc::TRY {
                    let x = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);
                    let u = format!("try, catch {}\n",i as i32+x);
                    s.push_str(&u);
                    i+=BCASIZE;
                }else if op==bc::TRYEND {
                    s.push_str("try end\n");
                    i+=BCSIZE;
                }else if op==bc::GETEXC {
                    s.push_str("get exception\n");
                    i+=BCSIZE;
                }else if op==bc::CRAISE {
                    s.push_str("raise further\n");
                    i+=BCSIZE;
                }else{
                    unreachable!("op ??");
                }
            },
            bc::APPLY => {s.push_str("apply\n"); i+=BCSIZE;}
            bc::HALT => {s.push_str("halt\n"); i+=BCSIZE;},
            _ => {unreachable!();}
        }
    }
    return s;
}

#[allow(dead_code)]
fn print_asm_listing(a: &[u32]){
    let s = asm_listing(a);
    println!("{}",&s);
}

#[allow(dead_code)]
fn print_data(a: &[Object]){
    println!("Data");
    for i in 0..a.len() {
        println!("[{}]: {}",i,a[i].to_repr());
    }
    if a.len()==0 {
        println!("empty\n");
    }else{
        println!();
    }
}

#[allow(dead_code)]
fn print_var_tab(vtab: &VarTab, indent: usize){
    let n = vtab.v.len();
    let a = &vtab.v[..];
    let mut sequent = false;

    print!("{:1$}","",indent);
    print!("context: ");
    for i in 0..n {
        if a[i].var_type == VarType::Context {
            if sequent {print!(", ")} else {sequent = true;}
            print!("{}",&a[i].s);
        }
    }
    println!();

    sequent = false;
    print!("{:1$}","",indent);
    print!("local: ");
    for i in 0..n {
        if a[i].var_type == VarType::Local {
            if sequent {print!(", ")} else {sequent = true;}
            print!("{}",&a[i].s);
        }
    }
    println!();

    sequent = false;
    print!("{:1$}","",indent);
    print!("argument: ");
    for i in 0..n {
        if a[i].var_type == VarType::Argument {
            if sequent {print!(", ")} else {sequent = true;}
            print!("{}",&a[i].s);
        }
    }
    println!();

    if let Some(ref context) = vtab.context {
        print_var_tab(context,indent+2);
    }
}

pub struct CompilerExtra{
    pub debug_mode: bool
}

pub fn compile(v: Vec<Token>, mode_cmd: bool, value: Value,
    history: &mut system::History, id: &str, rte: &Rc<RTE>
) -> Result<Rc<Module>,Error>
{
    let debug_mode = {
        if let Some(ref extra) = *rte.compiler_config.borrow() {
            extra.debug_mode
        }else{
            false
        }
    };
    let mut compilation = Compilation{
        mode_cmd: mode_cmd,
        syntax_nesting: 0, parens: 0, statement: !mode_cmd,
        history: history, file: id,
        pool: Pool{
            stab: HashMap::new(),
            stab_index: 0, data: Vec::new()
        },
        bv_blocks: Vec::new(),
        fn_indices: Vec::new(), vtab: VarTab::new(None),
        function_nesting: 0, jmp_stack: Vec::new(),
        coroutine: false, for_nesting: 0, debug_mode
    };
    let mut i = TokenIterator{index: 0, a: Rc::new(v.into_boxed_slice())};
    let y = compilation.ast(&mut i,value)?;
    // print_ast(&y,2);

    let mut bv: Vec<u32> = Vec::new();
    compilation.compile_ast(&mut bv, &y)?;
    push_bc(&mut bv, bc::HALT, y.line, y.col);
    let len = bv.len();
    compilation.offsets(&mut bv, len as i32);

    bv.append(&mut compilation.bv_blocks);

    // print_asm_listing(&bv);
    // print_data(&compilation.pool.data);
    let m = Rc::new(Module{
        // program: bv,
        program: Rc::from(bv),
        data: compilation.pool.data,
        rte: rte.clone(),
        gtab: rte.gtab.clone(),
        id: id.to_string()
    });
    return Ok(m);
}
