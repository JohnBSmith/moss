
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use std::ascii::AsciiExt;
use std::rc::Rc;
use std::collections::HashMap;
use std::mem::{transmute,replace};
use system;
use vm::{bc, BCSIZE, Module};
use object::{Object, U32String};

#[derive(Copy,Clone,PartialEq)]
enum SymbolType{
  Operator, Separator, Bracket,
  Bool, Int, Float, Imag,
  String, Identifier, Keyword
}

#[derive(Copy,Clone,PartialEq)]
enum Symbol{
  None, Plus, Minus, Ast, Div, Idiv, Mod, Pow,
  Lt, Gt, Le, Ge, Eq, Ne, In, Is, Isin, Notin, Isnot, Range,
  And, Or, Amp, Vline, Neg, Not, Tilde, Svert, Assignment,
  PLeft, PRight, BLeft, BRight, CLeft, CRight, Newline,
  Lshift, Rshift, Assert, Begin, Break, Catch, Continue,
  Elif, Else, End, For, Global, Goto, Label, Of,
  If, While, Do, Raise, Return, Sub, Table, Then, Try,
  Use, Yield, True, False, Null, Dot, Comma, Colon, Semicolon,
  List, Map, Application, Index, Block, Statement, Terminal
}

pub struct Token {
  token_type: SymbolType,
  value: Symbol,
  line: usize,
  col: usize,
  s: Option<String>
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
   KeywordsElement{s: "break",   t: &SymbolType::Keyword, v: &Symbol::Begin},
   KeywordsElement{s: "catch",   t: &SymbolType::Keyword, v: &Symbol::Catch},
   KeywordsElement{s: "continue",t: &SymbolType::Keyword, v: &Symbol::Continue},
   KeywordsElement{s: "do",      t: &SymbolType::Keyword, v: &Symbol::Do},
   KeywordsElement{s: "elif",    t: &SymbolType::Keyword, v: &Symbol::Elif},
   KeywordsElement{s: "else",    t: &SymbolType::Keyword, v: &Symbol::Else},
   KeywordsElement{s: "end",     t: &SymbolType::Keyword, v: &Symbol::End},
   KeywordsElement{s: "false",   t: &SymbolType::Bool,    v: &Symbol::False},
   KeywordsElement{s: "for",     t: &SymbolType::Keyword, v: &Symbol::For},
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
   KeywordsElement{s: "raise",   t: &SymbolType::Keyword, v: &Symbol::Raise},
   KeywordsElement{s: "return",  t: &SymbolType::Keyword, v: &Symbol::Return},
   KeywordsElement{s: "sub",     t: &SymbolType::Keyword, v: &Symbol::Sub},
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

fn compiler_error() -> !{
  panic!("compiler error");
}

fn is_keyword(id: &String) -> Option<&'static KeywordsElement> {
  let n: usize = KEYWORDS.len();
  for i in 0..n {
    if KEYWORDS[i].s==id  {return Some(&KEYWORDS[i]);}
  }
  return None;
}

pub fn scan(s: &str, line_start: usize, file: &str) -> Result<Vec<Token>,Error>{
  let mut v: Vec<Token> = Vec::new();
  let mut line=line_start;
  let mut col=1;
  let mut hcol: usize;
  let a: Vec<char> = s.chars().collect();
  let mut i=0;
  let n = a.len();
  while i<n {
    let c = a[i];
    if c.is_digit(10) {
      let j=i; hcol=col;
      let mut token_type = SymbolType::Int;
      while i<n {
        if a[i].is_digit(10) {
          i+=1; col+=1;
        }else if a[i]=='.' && token_type != SymbolType::Float{
          i+=1; col+=1;
          token_type = SymbolType::Float;
        }else if a[i]=='i' {
          token_type = SymbolType::Imag;
          break;
        }else{
          break;
        }
      }
      let number: &String = &a[j..i].iter().cloned().collect();
      if token_type == SymbolType::Imag {
        i+=1; col+=1;
      }
      v.push(Token{token_type: token_type,
        value: Symbol::None, line: line, col: hcol, s: Some(number.clone())});
    }else if (c.is_alphabetic() && c.is_ascii()) || a[i]=='_' {
      let j=i; hcol=col;
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
            line: line, col: hcol, s: None});
        },
        None => {
          v.push(Token{token_type: SymbolType::Identifier,
            value: Symbol::None, line: line, col: hcol, s: Some(id.clone())});
        }
      }
    }else{
      match c {
        ' ' | '\t' => {
          i+=1; col+=1;
        },
        '\n' => {
          v.push(Token{token_type: SymbolType::Separator,
            value: Symbol::Newline, line: line, col: col, s: None});
          i+=1; col=1; line+=1;
        },
        ',' => {
          v.push(Token{token_type: SymbolType::Separator,
            value: Symbol::Comma, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        ':' => {
          v.push(Token{token_type: SymbolType::Separator,
            value: Symbol::Colon, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        ';' => {
          v.push(Token{token_type: SymbolType::Separator,
            value: Symbol::Semicolon, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '(' => {
          v.push(Token{token_type: SymbolType::Bracket,
            value: Symbol::PLeft, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        ')' => {
          v.push(Token{token_type: SymbolType::Bracket,
            value: Symbol::PRight, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '[' => {
          v.push(Token{token_type: SymbolType::Bracket,
            value: Symbol::BLeft, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        ']' => {
          v.push(Token{token_type: SymbolType::Bracket,
            value: Symbol::BRight, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '{' => {
          v.push(Token{token_type: SymbolType::Bracket,
            value: Symbol::CLeft, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '}' => {
          v.push(Token{token_type: SymbolType::Bracket,
            value: Symbol::CRight, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '=' => {
          if i+1<n && a[i+1]=='=' {
            v.push(Token{token_type: SymbolType::Operator,
              value: Symbol::Eq, line: line, col: col, s: None});
            i+=2; col+=2;
          }else{
            v.push(Token{token_type: SymbolType::Operator,
              value: Symbol::Assignment, line: line, col: col, s: None});
            i+=1; col+=1;
          }
        },
        '+' => {
          v.push(Token{token_type: SymbolType::Operator,
            value: Symbol::Plus, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '-' => {
          v.push(Token{token_type: SymbolType::Operator,
            value: Symbol::Minus, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '*' => {
          v.push(Token{token_type: SymbolType::Operator,
            value: Symbol::Ast, line: line, col: col, s: None});
          i+=1; col+=1;
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
            v.push(Token{token_type: SymbolType::Operator,
              value: Symbol::Idiv, line: line, col: col, s: None});
            i+=2; col+=2;
          }else{
            v.push(Token{token_type: SymbolType::Operator,
              value: Symbol::Div, line: line, col: col, s: None});
            i+=1; col+=1;
          }
        },
        '%' => {
          v.push(Token{token_type: SymbolType::Operator,
            value: Symbol::Mod, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '^' => {
          v.push(Token{token_type: SymbolType::Operator,
            value: Symbol::Pow, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '.' => {
          if i+1<n && a[i+1]=='.' {
            v.push(Token{token_type: SymbolType::Operator,
              value: Symbol::Range, line: line, col: col, s: None});
            i+=2; col+=2;
          }else{
            v.push(Token{token_type: SymbolType::Operator,
              value: Symbol::Dot, line: line, col: col, s: None});
            i+=1; col+=1;
          }
        },
        '<' => {
          if i+1<n && a[i+1]=='=' {
            v.push(Token{token_type: SymbolType::Operator,
              value: Symbol::Le, line: line, col: col, s: None});
            i+=2; col+=2;
          }else if i+1<n && a[i+1]=='<' {
            v.push(Token{token_type: SymbolType::Operator,
              value: Symbol::Lshift, line: line, col: col, s: None});
            i+=2; col+=2;
          }else{
            v.push(Token{token_type: SymbolType::Operator,
              value: Symbol::Lt, line: line, col: col, s: None});
            i+=1; col+=1;
          }
        },
        '>' => {
          if i+1<n && a[i+1]=='=' {
            v.push(Token{token_type: SymbolType::Operator,
              value: Symbol::Ge, line: line, col: col, s: None});
            i+=2; col+=2;
          }else if i+1<n && a[i+1]=='>' {
            v.push(Token{token_type: SymbolType::Operator,
              value: Symbol::Rshift, line: line, col: col, s: None});
            i+=2; col+=2;            
          }else{
            v.push(Token{token_type: SymbolType::Operator,
              value: Symbol::Gt, line: line, col: col, s: None});
            i+=1; col+=1;
          }
        },
        '|' => {
          v.push(Token{token_type: SymbolType::Operator,
            value: Symbol::Vline, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '&' => {
          v.push(Token{token_type: SymbolType::Operator,
            value: Symbol::Amp, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '$' => {
          v.push(Token{token_type: SymbolType::Operator,
            value: Symbol::Svert, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '~' => {
          v.push(Token{token_type: SymbolType::Operator,
            value: Symbol::Tilde, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '"' => {
          hcol=col;
          i+=1; col+=1;
          let j=i;
          while i<n && a[i]!='"' {i+=1; col+=1;}
          let s: &String = &a[j..i].iter().cloned().collect();
          v.push(Token{token_type: SymbolType::String,
            value: Symbol::None, line: line, col: hcol, s: Some(s.clone())
          });
          i+=1; col+=1;
        },
        '\'' => {
          hcol=col;
          i+=1; col+=1;
          let j=i;
          while i<n && a[i]!='\'' {i+=1; col+=1;}
          let s: &String = &a[j..i].iter().cloned().collect();
          v.push(Token{token_type: SymbolType::String,
            value: Symbol::None, line: line, col: hcol, s: Some(s.clone())
          });
          i+=1; col+=1;
        },
        '!' => {
          if i+1<n && a[i+1]=='=' {
            v.push(Token{token_type: SymbolType::Operator,
              value: Symbol::Ne, line: line, col: col, s: None});
            i+=2; col+=2;
          }else{
            return Err(Box::new(EnumError::Syntax(SyntaxError{line: line, col: col, file: String::from(file),
              s: format!("unexpected character '{}'.", c)})));
          }
        },
        '#' => {
          while i<n && a[i]!='\n' {i+=1; col+=1;}
          v.push(Token{token_type: SymbolType::Separator,
            value: Symbol::Newline, line: line, col: col, s: None});
          i+=1; col=1; line+=1;
        },
        _ => {
          return Err(Box::new(EnumError::Syntax(SyntaxError{line: line, col: col, file: String::from(file),
            s: format!("unexpected character '{}'.", c)})));
        }
      }
    }
  }
  v.push(Token{token_type: SymbolType::Separator,
    value: Symbol::Terminal, line: line, col: col, s: None});
  return Ok(v);
}

fn symbol_to_string(value: Symbol) -> &'static str {
  return match value {
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
    Symbol::Block => "block", Symbol::Statement => "statement",
    Symbol::Range => "..",
    Symbol::Assignment => "=",
    Symbol::Newline => "\\n",
    Symbol::Assert => "assert",
    Symbol::Begin => "begin",
    Symbol::Break => "break",
    Symbol::Catch => "catch",
    Symbol::Continue => "continue",
    Symbol::Elif => "elif",
    Symbol::Else => "else",
    Symbol::End => "end",
    Symbol::False => "false",
    Symbol::For => "for",
    Symbol::Global => "global",
    Symbol::Goto => "goto",
    Symbol::If => "if",
    Symbol::Null => "null",
    Symbol::Of => "of",
    Symbol::Raise => "raise",
    Symbol::Return => "return",
    Symbol::Sub => "sub",
    Symbol::Table => "table",
    Symbol::Then => "then",
    Symbol::True => "true",
    Symbol::Try => "try",
    Symbol::Use => "use",
    Symbol::While => "while",
    Symbol::Yield => "yield",
    Symbol::Terminal => "terminal",
    _ => "unknown token value"
  };
}

fn print_token(x: &Token){
  match x.token_type {
    SymbolType::Identifier | SymbolType::Int | SymbolType::Float => {
      print!("[{}]",match x.s {Some(ref s) => s, None => compiler_error()});
    },
    SymbolType::Imag => {
      print!("[{}i]",match x.s {Some(ref s) => s, None => compiler_error()});    
    },
    SymbolType::String => {
      print!("[\"{}\"]",match x.s {Some(ref s) => s, None => compiler_error()});    
    }
    SymbolType::Operator | SymbolType::Separator |
    SymbolType::Bracket  | SymbolType::Keyword | SymbolType::Bool => {
      print!("[{}]",symbol_to_string(x.value));
    }
  }
}

pub fn print_vtoken(v: &Vec<Token>){
  for x in v {print_token(x);}
  println!();
}

fn print_ast(t: &ASTNode, indent: usize){
  print!("{:1$}","",indent);
  match t.symbol_type {
    SymbolType::Identifier | SymbolType::Int | SymbolType::Float => {
      println!("{}",match t.s {Some(ref s) => s, None => compiler_error()});
    },
    SymbolType::Imag => {
      println!("{}i",match t.s {Some(ref s) => s, None => compiler_error()});    
    },
    SymbolType::String => {
      println!("\"{}\"",match t.s {Some(ref s) => s, None => compiler_error()});    
    },
    SymbolType::Operator | SymbolType::Separator |
    SymbolType::Keyword  | SymbolType::Bool => {
      if t.value == Symbol::Sub {
        match t.s {
          Some(ref s) => {println!("sub {}",s);},
          None => {println!("sub");}
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

fn scan_line(line_start: usize, h: &mut system::History) -> Result<Vec<Token>,Error>{
  let input = match system::getline_history("| ",h) {
    Ok(x) => x,
    Err(x) => panic!()
  };
  h.append(&input);
  return scan(&input,line_start,"command line");
}

enum ComplexInfoA{
}

enum Info{
  None, Some(u8), A(Box<ComplexInfoA>)
}

struct ASTNode{
  line: usize, col: usize,
  symbol_type: SymbolType,
  value: Symbol,
  info: Info,
  s: Option<String>,
  a: Option<Box<[Rc<ASTNode>]>>
}

#[derive(Clone,Copy)]
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
  fn_id: Option<String>
}
impl VarTab{
  fn new(id: Option<String>) -> VarTab {
    VarTab{v: Vec::new(),
      count_local: 0, count_arg: 0, count_context: 0, count_global: 0,
      context: None, fn_id: id
    }
  }
}

pub struct Compilation<'a>{
  mode_cmd: bool,
  index: usize,
  parens: bool,
  statement: bool,
  history: &'a mut system::History,
  file: &'a str,
  stab: HashMap<String,usize>,
  stab_index: usize,
  data: Vec<Object>,
  bv_blocks: Vec<u8>,
  fn_indices: Vec<usize>,
  vtab: VarTab,
  function_nesting: usize
}

struct TokenIterator{
  pub a: Rc<Box<[Token]>>,
  pub index: usize
}

impl TokenIterator{
  fn next_token(&mut self, c: &mut Compilation) -> Result<Rc<Box<[Token]>>,Error>{
    if c.mode_cmd {
      let value = self.a[self.index].value;
      if value==Symbol::Terminal {
        let line = self.a[self.index].line;
        let v = try!(scan_line(line+1,c.history));
        self.a = Rc::new(v.into_boxed_slice());
        self.index=0;
        return self.next_token(c);
      }
    }
    return Ok(self.a.clone());
  }
  fn next_any_token(&mut self, c: &mut Compilation) -> Result<Rc<Box<[Token]>>,Error>{
    if c.parens {
      return self.next_token(c);
    }else{
      return Ok(self.a.clone());
    }
  }
}

fn unary_operator(line: usize, col: usize,
  value: Symbol, x: Rc<ASTNode>) -> Rc<ASTNode>
{
  return Rc::new(ASTNode{line: line, col: col, symbol_type: SymbolType::Operator,
    value: value, info: Info::None, s: None, a: Some(Box::new([x]))}); 
}

fn binary_operator(line: usize, col: usize, value: Symbol,
  x: Rc<ASTNode>, y: Rc<ASTNode>) -> Rc<ASTNode>
{
  return Rc::new(ASTNode{line: line, col: col, symbol_type: SymbolType::Operator,
    value: value, info: Info::None, s: None, a: Some(Box::new([x,y]))}); 
}

fn atomic_literal(line: usize, col: usize, value: Symbol) -> Rc<ASTNode>{
  return Rc::new(ASTNode{line: line, col: col, symbol_type: SymbolType::Keyword,
    value: value, info: Info::None, s: None, a: None});
}

impl<'a> Compilation<'a>{

fn syntax_error(&self, line: usize, col: usize, s: String) -> Error{
  Box::new(EnumError::Syntax(SyntaxError{line: line, col: col,
    file: String::from(self.file), s: s
  }))
}

fn unexpected_token(&mut self, line: usize, col: usize, value: Symbol) -> Error{
  Box::new(EnumError::Syntax(SyntaxError{line: line, col: col, file: String::from(self.file),
    s: format!("unexpected token: '{}'.",symbol_to_string(value))
  }))
}

fn list_literal(&mut self, i: &mut TokenIterator) -> Result<Box<[Rc<ASTNode>]>,Error> {
  let mut v: Vec<Rc<ASTNode>> = Vec::new();
  let p = try!(i.next_token(self));
  let t = &p[i.index];
  if t.value==Symbol::BRight {
    i.index+=1;
    return Ok(v.into_boxed_slice());
  }
  loop{
    let x = try!(self.expression(i));
    v.push(x);
    let p = try!(i.next_token(self));
    let t = &p[i.index];
    if t.value==Symbol::Comma {
      i.index+=1;
      let p = try!(i.next_token(self));
      let t = &p[i.index];
      if t.value==Symbol::BRight {
        i.index+=1;
        break;
      }
    }else if t.value==Symbol::BRight {
      i.index+=1;
      break;
    }else{
      return Err(self.syntax_error(t.line, t.col, String::from("expected ',' or ']'.")));
    }
  }
  return Ok(v.into_boxed_slice());
}

fn map_literal(&mut self, i: &mut TokenIterator) -> Result<Box<[Rc<ASTNode>]>,Error> {
  let mut v: Vec<Rc<ASTNode>> = Vec::new();
  let p = try!(i.next_token(self));
  let t = &p[i.index];
  if t.value == Symbol::CRight {
    i.index+=1;
    return Ok(v.into_boxed_slice());
  }
  loop{
    let key = try!(self.expression(i));
    let p = try!(i.next_token(self));
    let t = &p[i.index];
    if t.value == Symbol::Comma {
      let value = atomic_literal(t.line, t.col, Symbol::Null);
      v.push(key);
      v.push(value);
      i.index+=1;
    }else if t.value == Symbol::CRight {
      let value = atomic_literal(t.line, t.col, Symbol::Null);
      v.push(key);
      v.push(value);
      i.index+=1;
      break;
    }else if t.value == Symbol::Colon {
      i.index+=1;
      let value = try!(self.expression(i));
      let p2 = try!(i.next_token(self));
      let t2 = &p2[i.index];
      v.push(key);
      v.push(value);
      if t2.value == Symbol::CRight {
        i.index+=1;
        break;
      }else if t2.value != Symbol::Comma {
        return Err(self.syntax_error(t2.line, t2.col, String::from("expected ',' or '}'.")));
      }
      i.index+=1;
    }else if t.value== Symbol::Assignment {
      i.index+=1;
      if key.symbol_type != SymbolType::Identifier {
        return Err(self.syntax_error(t.line, t.col, String::from("expected an identifier before '='.")));
      }
      let value = try!(self.expression(i));
      let skey = Rc::new(ASTNode{
        line: key.line, col: key.col,
        symbol_type: SymbolType::String, value: Symbol::None,
        info: Info::None, s: key.s.clone(), a: None
      });
      v.push(skey);
      v.push(value);
      let p2 = try!(i.next_token(self));
      let t2 = &p2[i.index];
      if t2.value == Symbol::CRight {
        i.index+=1;
        break;
      }else if t2.value != Symbol::Comma {
        return Err(self.syntax_error(t2.line, t2.col, String::from("expected ',' or '}'.")));
      }
      i.index+=1;
    }else{
      return Err(self.syntax_error(t.line, t.col, String::from("expected ',' or '=' or ':' or '}'.")));
    }
  }
  return Ok(v.into_boxed_slice());
}

fn table_literal(&mut self, i: &mut TokenIterator, t0: &Token) -> Result<Rc<ASTNode>,Error> {
  let mut v: Vec<Rc<ASTNode>> = Vec::new();
  let p = try!(i.next_token(self));
  let t = &p[i.index];
  if t.value==Symbol::BLeft {
    i.index+=1;
    let prototype = try!(self.expression(i));
    v.push(prototype);
    let p2 = try!(i.next_token(self));
    let t2 = &p2[i.index];
    if t2.value != Symbol::BRight {
      return Err(self.syntax_error(t2.line, t2.col, String::from("expected ',' or ']'.")));
    }
    i.index+=1;
  }else{
    v.push(atomic_literal(t0.line, t0.col, Symbol::Null));
  }
  loop{
    let key = try!(self.expression(i));
    let p = try!(i.next_token(self));
    let t = &p[i.index];
    if t.value == Symbol::Colon {
      i.index+=1;
      let value = try!(self.expression(i));
      v.push(key);
      v.push(value);
    }else if t.value == Symbol::Assignment {
      i.index+=1;
      let value = try!(self.expression(i));
      let skey = Rc::new(ASTNode{
        line: key.line, col: key.col,
        symbol_type: SymbolType::String, value: Symbol::None,
        info: Info::None, s: key.s.clone(), a: None
      });
      v.push(skey);
      v.push(value);
    }else{ 
      return Err(self.syntax_error(t.line, t.col, String::from("expected ':' or '='.")));
    }
    let p = try!(i.next_token(self));
    let t = &p[i.index];
    if t.value == Symbol::End {
      i.index+=1;
      break;
    }else if t.value == Symbol::Comma {
      i.index+=1;
    }else{
      return Err(self.syntax_error(t.line, t.col, String::from("expected ',' or 'end'.")));
    }
  }
  return Ok(Rc::new(ASTNode{line: t0.line, col: t0.col, symbol_type: SymbolType::Keyword,
    value: Symbol::Table, info: Info::None, s: None, a: Some(v.into_boxed_slice())}));
}

fn arguments_list(&mut self, i: &mut TokenIterator, t0: &Token)
  -> Result<Rc<ASTNode>,Error>
{
  let mut v: Vec<Rc<ASTNode>> = Vec::new();
  let p = try!(i.next_token(self));
  let t = &p[i.index];
  if t.value == Symbol::Vline {
    i.index+=1;
  }else{
    loop{
      let x = try!(self.atom(i));
      v.push(x);
      let p = try!(i.next_token(self));
      let t = &p[i.index];
      if t.value == Symbol::Comma {
        i.index+=1;
        let p = try!(i.next_token(self));
        let t = &p[i.index];
        if t.value==Symbol::Vline {
          i.index+=1;
          break;
        }
      }else if t.value==Symbol::Vline {
        i.index+=1;
        break;
      }else{
        return Err(self.syntax_error(t.line, t.col, String::from("expected ',' or '|'.")));
      }
    }
  }
  return Ok(Rc::new(ASTNode{line: t0.line, col: t0.col,
    symbol_type: SymbolType::Operator, value: Symbol::List,
    info: Info::None, s: None, a: Some(v.into_boxed_slice())
  }));
}

fn concise_function_literal(&mut self, i: &mut TokenIterator, t0: &Token)
  -> Result<Rc<ASTNode>,Error>
{
  let args = try!(self.arguments_list(i,t0));
  let x = try!(self.expression(i));
  return Ok(Rc::new(ASTNode{line: t0.line, col: t0.col,
    symbol_type: SymbolType::Keyword, value: Symbol::Sub,
    info: Info::None, s: None, a: Some(Box::new([args,x]))
  }));
}

fn function_literal(&mut self, i: &mut TokenIterator, t0: &Token)
  -> Result<Rc<ASTNode>,Error>
{
  let p = try!(i.next_token(self));
  let t = &p[i.index];
  let id = if t.token_type == SymbolType::Identifier {
    i.index+=1;
    t.s.clone()
  }else{
    None
  };
  let p = try!(i.next_token(self));
  let t = &p[i.index];
  if t.value != Symbol::Vline {
    return Err(self.syntax_error(t.line, t.col, String::from("expected '|'.")));
  }
  i.index+=1;
  let args = try!(self.arguments_list(i,t0));

  let statement = self.statement;
  self.statement = true;
  self.function_nesting+=1;
  let x = try!(self.statements(i,true));
  self.function_nesting-=1;
  self.statement = statement;

  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value != Symbol::End {
    return Err(self.syntax_error(t.line, t.col, String::from("expected 'end'.")));
  }
  i.index+=1;
  try!(self.end_of(i,Symbol::Sub));
  return Ok(Rc::new(ASTNode{line: t0.line, col: t0.col,
    symbol_type: SymbolType::Keyword, value: Symbol::Sub,
    info: Info::None, s: id, a: Some(Box::new([args,x]))
  }));
}


fn application(&mut self, i: &mut TokenIterator, f: Rc<ASTNode>, terminal: Symbol)
  -> Result<Rc<ASTNode>,Error>
{
  let mut v: Vec<Rc<ASTNode>> = Vec::new();
  let line = f.line;
  let col = f.col;
  v.push(f);
  let p = try!(i.next_token(self));
  let t = &p[i.index];
  if t.value == terminal {
    i.index+=1;
  }else{
    loop{
      let x = try!(self.expression(i));
      v.push(x);
      let p = try!(i.next_token(self));
      let t = &p[i.index];
      if t.value == Symbol::Comma {
        i.index+=1;
      }else if t.value == terminal {
        i.index+=1;
        break;
      }else{
        return Err(self.unexpected_token(t.line, t.col, t.value));
      }
    }
  }
  let value = if terminal==Symbol::PRight {
    Symbol::Application
  }else{
    Symbol::Index
  };
  return Ok(Rc::new(ASTNode{line: line, col: col, symbol_type: SymbolType::Operator,
    value: value, info: Info::None, s: None, a: Some(v.into_boxed_slice())}));
}

fn atom(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error> {
  let p = try!(i.next_token(self));
  let t = &p[i.index];
  let mut y;
  if t.token_type==SymbolType::Identifier || t.token_type==SymbolType::Int ||
     t.token_type==SymbolType::String || t.token_type==SymbolType::Float ||
     t.token_type==SymbolType::Imag
  {
    i.index+=1;
    y = Rc::new(ASTNode{line: t.line, col: t.col, symbol_type: t.token_type,
      value: t.value, info: Info::None, s: t.s.clone(), a: None});
  }else if t.value==Symbol::PLeft {
    i.index+=1;
    self.parens = true;
    y = try!(self.expression(i));
    let p = try!(i.next_token(self));
    let t = &p[i.index];
    self.parens = false;
    if t.value != Symbol::PRight {
      return Err(self.syntax_error(t.line, t.col, String::from("expected ')'.")));
    }
    i.index+=1;
  }else if t.value==Symbol::BLeft {
    i.index+=1;
    let x = try!(self.list_literal(i));
    y = Rc::new(ASTNode{line: t.line, col: t.col,
      symbol_type: SymbolType::Operator, value: Symbol::List,
      info: Info::None, s: None, a: Some(x)
    });
  }else if t.value==Symbol::CLeft {
    i.index+=1;
    let x = try!(self.map_literal(i));
    y = Rc::new(ASTNode{line: t.line, col: t.col,
      symbol_type: SymbolType::Operator, value: Symbol::Map,
      info: Info::None, s: None, a: Some(x)
    });
  }else if t.value==Symbol::Null ||
    t.value==Symbol::False || t.value==Symbol::True
  {
    i.index+=1;
    y = atomic_literal(t.line, t.col, t.value);
  }else if t.value==Symbol::Table {
    i.index+=1;
    y = try!(self.table_literal(i,t));
  }else if t.value==Symbol::Vline {
    i.index+=1;
    y = try!(self.concise_function_literal(i,t));
  }else if t.value==Symbol::Sub {
    i.index+=1;
    y = try!(self.function_literal(i,t));
  }else{
    return Err(self.unexpected_token(t.line, t.col, t.value));
  }
  loop{
    let p2 = try!(i.next_any_token(self));
    let t2 = &p2[i.index];
    if t2.value == Symbol::PLeft {
      i.index+=1;
      y = try!(self.application(i,y,Symbol::PRight));
    }else if t2.value == Symbol::BLeft {
      i.index+=1;
      y = try!(self.application(i,y,Symbol::BRight));
    }else{
      return Ok(y);
    }
  }
}

fn power(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  let x = try!(self.atom(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value==Symbol::Pow {
    i.index+=1;
    let y = try!(self.power(i));
    return Ok(binary_operator(t.line,t.col,Symbol::Pow,x,y));
  }else{
    return Ok(x);
  }
}

fn signed_expression(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  let p = try!(i.next_token(self));
  let t = &p[i.index];
  if t.value==Symbol::Minus || t.value==Symbol::Tilde {
    i.index+=1;
    let x = try!(self.power(i));
    let value = if t.value==Symbol::Minus
      {Symbol::Neg} else {Symbol::Tilde};
    return Ok(unary_operator(t.line,t.col,value,x));
  }else if t.value==Symbol::Plus {
    i.index+=1;
    return self.power(i);
  }else{
    return self.power(i);
  }
}

fn factor(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  let mut y = try!(self.signed_expression(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  let value=t.value;
  if value==Symbol::Ast || value==Symbol::Div ||
     value==Symbol::Mod || value==Symbol::Idiv
  {
    i.index+=1;
    let x = try!(self.signed_expression(i));
    y = binary_operator(t.line,t.col,value,y,x);
    loop{
      let p = try!(i.next_any_token(self));
      let t = &p[i.index];
      let value = t.value;
      if value != Symbol::Ast && value != Symbol::Div &&
         value != Symbol::Mod && value != Symbol::Idiv
      {
        return Ok(y);
      }
      i.index+=1;
      let x = try!(self.signed_expression(i));
      y = binary_operator(t.line,t.col,value,y,x);  
    }
  }else{
    return Ok(y);
  }
}

fn addition(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  let mut y = try!(self.factor(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  let value=t.value;
  if value==Symbol::Plus || value==Symbol::Minus {
    i.index+=1;
    let x = try!(self.factor(i));
    y = binary_operator(t.line,t.col,value,y,x);
    loop{
      let p = try!(i.next_any_token(self));
      let t = &p[i.index];
      let value=t.value;
      if value!=Symbol::Plus && value!=Symbol::Minus {
        return Ok(y);
      }
      i.index+=1;
      let x = try!(self.factor(i));
      y = binary_operator(t.line,t.col,value,y,x);  
    }
  }else{
    return Ok(y);
  }
}

fn shift(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  let x = try!(self.addition(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value==Symbol::Lshift || t.value==Symbol::Rshift {
    i.index+=1;
    let y = try!(self.addition(i));
    return Ok(binary_operator(t.line,t.col,t.value,x,y));
  }else{
    return Ok(x);
  }
}

fn intersection(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  let mut y = try!(self.shift(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  let value=t.value;
  if value==Symbol::Amp {
    i.index+=1;
    let x = try!(self.shift(i));
    y = binary_operator(t.line,t.col,value,y,x);
    loop{
      let p = try!(i.next_any_token(self));
      let t = &p[i.index];
      let value=t.value;
      if value!=Symbol::Amp {
        return Ok(y);
      }
      i.index+=1;
      let x = try!(self.shift(i));
      y = binary_operator(t.line,t.col,value,y,x);  
    }
  }else{
    return Ok(y);
  }
}

fn union(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  let mut y = try!(self.intersection(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  let value=t.value;
  if value==Symbol::Vline || value==Symbol::Svert {
    i.index+=1;
    let x = try!(self.intersection(i));
    y = binary_operator(t.line,t.col,value,y,x);
    loop{
      let p = try!(i.next_any_token(self));
      let t = &p[i.index];
      let value=t.value;
      if value != Symbol::Vline && value != Symbol::Svert {
        return Ok(y);
      }
      i.index+=1;
      let x = try!(self.intersection(i));
      y = binary_operator(t.line,t.col,value,y,x);  
    }
  }else{
    return Ok(y);
  }
}

fn range(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  let x = try!(self.union(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value==Symbol::Range {
    i.index+=1;
    let y = try!(self.union(i));
    let p2 = try!(i.next_any_token(self));
    let t2 = &p2[i.index];
    if t2.value==Symbol::Colon {
      i.index+=1;
      let d = try!(self.union(i));
      return Ok(Rc::new(ASTNode{
        symbol_type: SymbolType::Operator, value: Symbol::Range,
        line: t2.line, col: t2.col, info: Info::None,
        s: None, a: Some(Box::new([x,y,d]))
      }));
    }else{
      return Ok(binary_operator(t.line,t.col,t.value,x,y));
    }
  }else{
    return Ok(x);
  }
}

fn comparison(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  let x = try!(self.range(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  let value=t.value;
  if value==Symbol::Lt || value==Symbol::Gt ||
     value==Symbol::Le || value==Symbol::Ge
  {
    i.index+=1;
    let y = try!(self.range(i));
    return Ok(binary_operator(t.line,t.col,value,x,y));
  }else{
    return Ok(x);
  }
}

fn eq_expression(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  let x = try!(self.comparison(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  let value=t.value;
  if value==Symbol::Eq || value==Symbol::Ne ||
     value==Symbol::Is || value==Symbol::Isin ||
     value==Symbol::In || value==Symbol::Notin
  {
    i.index+=1;
    let y = try!(self.comparison(i));
    return Ok(binary_operator(t.line,t.col,value,x,y));
  }else{
    return Ok(x);
  }
}

fn negation(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  let p = try!(i.next_token(self));
  let t = &p[i.index];
  if t.value==Symbol::Not {
    i.index+=1;
    let x = try!(self.eq_expression(i));
    return Ok(unary_operator(t.line,t.col,Symbol::Not,x));
  }else{
    return self.eq_expression(i);
  }
}

fn conjunction(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  let x = try!(self.negation(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value==Symbol::And {
    i.index+=1;
    let y = try!(self.negation(i));
    return Ok(binary_operator(t.line,t.col,Symbol::And,x,y));
  }else{
    return Ok(x);
  }
}

fn disjunction(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  let x = try!(self.conjunction(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value==Symbol::Or {
    i.index+=1;
    let y = try!(self.conjunction(i));
    return Ok(binary_operator(t.line,t.col,Symbol::Or,x,y));
  }else{
    return Ok(x);
  }
}

fn if_expression(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  let x = try!(self.disjunction(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value==Symbol::If {
    i.index+=1;
    let condition = try!(self.expression(i));
    let p2 = try!(i.next_any_token(self));
    let t2 = &p[i.index];
    if t2.value==Symbol::Else {
      i.index+=1;
      let y = try!(self.expression(i));
      return Ok(Rc::new(ASTNode{
        line: t.line, col: t.col, symbol_type: SymbolType::Operator,
        value: Symbol::If, info: Info::None,
        s: None, a: Some(Box::new([condition,x,y]))
      }));
    }else{
      return Ok(binary_operator(t.line,t.col,Symbol::If,condition,x));
    }
  }else{
    return Ok(x);
  }
}

fn expression(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  return self.if_expression(i);
}

fn assignment(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  let x = try!(self.expression(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value==Symbol::Assignment {
    i.index+=1;
    let y = try!(self.expression(i));
    return Ok(binary_operator(t.line,t.col,Symbol::Assignment,x,y));
  }else if self.statement {
    return Ok(Rc::new(ASTNode{line: t.line, col: t.col, symbol_type: SymbolType::Keyword,
      value: Symbol::Statement, info: Info::None, s: None, a: Some(Box::new([x]))}));
  }else{
    return Ok(x);
  }
}

fn while_statement(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  let condition = try!(self.expression(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value == Symbol::Do || t.value == Symbol::Newline {
    i.index+=1;
  }else{
    return Err(self.syntax_error(t.line, t.col, String::from("expected 'do' or a line break.")));
  }
  let body = try!(self.statements(i,false));
  return Ok(Rc::new(ASTNode{line: t.line, col: t.col, symbol_type: SymbolType::Keyword,
    value: Symbol::While, info: Info::None, s: None, a: Some(Box::new([condition,body]))}));  
}

fn for_statement(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  let variable = try!(self.atom(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value != Symbol::In {
    return Err(self.syntax_error(t.line, t.col, String::from("expected 'in'.")));
  }
  i.index+=1;
  let a = try!(self.expression(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value == Symbol::Do || t.value == Symbol::Newline {
    i.index+=1;
  }else{
    return Err(self.syntax_error(t.line, t.col, String::from("expected 'do' or a line break.")));
  }
  let body = try!(self.statements(i,false));
  return Ok(Rc::new(ASTNode{line: t.line, col: t.col, symbol_type: SymbolType::Keyword,
    value: Symbol::For, info: Info::None, s: None, a: Some(Box::new([variable,a,body]))}));  
}

fn if_statement(&mut self, i: &mut TokenIterator, t0: &Token) -> Result<Rc<ASTNode>,Error>{
  let mut v: Vec<Rc<ASTNode>> = Vec::new();
  let condition = try!(self.expression(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value == Symbol::Then || t.value == Symbol::Newline {
    i.index+=1;
  }else{
    return Err(self.syntax_error(t.line, t.col, String::from("expected 'then' or a line break.")));
  }
  let body = try!(self.statements(i,false));
  v.push(condition);
  v.push(body);
  loop{
    let p = try!(i.next_any_token(self));
    let t = &p[i.index];
    if t.value == Symbol::Elif {
      i.index+=1;
      let condition = try!(self.expression(i));
      let p = try!(i.next_any_token(self));
      let t = &p[i.index];
      if t.value == Symbol::Then || t.value == Symbol::Newline {
        i.index+=1;
      }else{
        return Err(self.syntax_error(t.line, t.col, String::from("expected 'then' or a line break.")));
      }
      let body = try!(self.statements(i,false));
      v.push(condition);
      v.push(body);
    }else if t.value == Symbol::Else {
      i.index+=1;
      let body = try!(self.statements(i,false));
      v.push(body);
    }else if t.value == Symbol::End {
      break;
    }
  }
  return Ok(Rc::new(ASTNode{line: t0.line, col: t0.col, symbol_type: SymbolType::Keyword,
    value: Symbol::If, info: Info::None, s: None, a: Some(v.into_boxed_slice())}));
}

fn end_of(&mut self, i: &mut TokenIterator, symbol: Symbol) -> Result<(),Error>{
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value == Symbol::Of {
    i.index+=1;
    let p = try!(i.next_any_token(self));
    let t = &p[i.index];
    if t.value != symbol {
      return Err(self.syntax_error(t.line, t.col,
        format!("expected 'end of {}', but got 'end of {}'.",
          symbol_to_string(symbol),
          symbol_to_string(t.value))
      ));
    }
    i.index+=1;
  }
  return Ok(());
}

fn statements(&mut self, i: &mut TokenIterator, last_value: bool)
  -> Result<Rc<ASTNode>,Error>
{
  let mut v: Vec<Rc<ASTNode>> = Vec::new();
  let p0 = try!(i.next_any_token(self));
  let t0 = &p0[i.index];
  loop{
    let p = try!(i.next_any_token(self));
    let t = &p[i.index];
    if t.value == Symbol::Newline {
      i.index+=1;
      continue;
    }else if t.value == Symbol::Terminal {
      break;
    }
    if t.value == Symbol::While {
      i.index+=1;
      let statement = self.statement;
      self.statement = true;
      let x = try!(self.while_statement(i));
      self.statement = statement;
      v.push(x);
      let p = try!(i.next_any_token(self));
      let t = &p[i.index];
      if t.value != Symbol::End {    
        return Err(self.syntax_error(t.line, t.col, String::from("expected 'end'.")));
      }
      i.index+=1;
      try!(self.end_of(i,Symbol::While));
    }else if t.value == Symbol::For {
      i.index+=1;
      let statement = self.statement;
      self.statement = true;
      let x = try!(self.for_statement(i));
      self.statement = statement;
      v.push(x);
      let p = try!(i.next_any_token(self));
      let t = &p[i.index];
      if t.value != Symbol::End {
        return Err(self.syntax_error(t.line, t.col, String::from("expected 'end'.")));    
      }
      i.index+=1;
      try!(self.end_of(i,Symbol::For));
    }else if t.value == Symbol::If {
      i.index+=1;
      let statement = self.statement;
      self.statement = true;
      let x = try!(self.if_statement(i,t));
      self.statement = statement;
      v.push(x);
      let p = try!(i.next_any_token(self));
      let t = &p[i.index];
      if t.value != Symbol::End {
        return Err(self.syntax_error(t.line, t.col, String::from("expected 'end'.")));
      }
      i.index+=1;
      try!(self.end_of(i,Symbol::If));
    }else if t.value == Symbol::End || t.value == Symbol::Elif ||
      t.value == Symbol::Else
    {
      break;
    }else{
      let x = try!(self.assignment(i));
      v.push(x);
    }
    let p = try!(i.next_any_token(self));
    let t = &p[i.index];
    if t.value == Symbol::Semicolon {
      i.index+=1;
    }else if t.value == Symbol::End || t.value == Symbol::Elif ||
      t.value == Symbol::Else
    {
      break;
    }else if t.value == Symbol::Newline {
      i.index+=1;
    }else if t.value == Symbol::Terminal {
      break;
    }else{
      return Err(self.unexpected_token(t.line, t.col, t.value));
    }
  }
  if last_value {
    let n = v.len();
    if n>0 && v[n-1].value == Symbol::Statement {
      let x = ast_argv(&v[n-1])[0].clone();
      v[n-1] = x;
    }else{
      v.push(atomic_literal(t0.line,t0.col,Symbol::Null));
    }
  }
  if v.len()==1 {
    return Ok(match v.pop() {Some(x) => x, None => compiler_error()});
  }else{
    return Ok(Rc::new(ASTNode{line: t0.line, col: t0.col, symbol_type: SymbolType::Keyword,
      value: Symbol::Block, info: Info::None, s: None, a: Some(v.into_boxed_slice())}));
  }
}

fn ast(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,Error>{
  return self.statements(i,true);
}

fn compile_operator(&mut self, v: &mut Vec<u8>, t: &Rc<ASTNode>, byte_code: u8) -> Result<(),Error>{
  let a = ast_argv(t);
  for i in 0..a.len() {
    try!(self.compile_ast(v,&a[i]));
  }
  v.push(byte_code);
  push_line_col(v,t.line,t.col);
  return Ok(());
}

fn get_index(&mut self, key: &str) -> usize{
  if let Some(index) = self.stab.get(key) {
    return *index;
  }
  self.data.push(U32String::new_object_str(key));
  self.stab.insert(String::from(key),self.stab_index);
  self.stab_index+=1;
  return self.stab_index-1;
}

// while c do b end
// (1) c JPZ[2] b JMP[1] (2)
fn compile_while(&mut self, v: &mut Vec<u8>, t: &Rc<ASTNode>)
  -> Result<(),Error>
{
  let index1 = v.len();
  let a = ast_argv(t);
  try!(self.compile_ast(v,&a[0]));
  push_bc(v,bc::JZ,t.line,t.col);
  let index2 = v.len();
  push_u32(v,0xcafe);
  try!(self.compile_ast(v,&a[1]));
  push_bc(v,bc::JMP,t.line,t.col);
  let len = v.len();
  push_i32(v,(BCSIZE+index1) as i32-len as i32);
  let len = v.len();
  write_i32(&mut v[index2..index2+4],(BCSIZE+len) as i32-index2 as i32);
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

fn compile_if(&mut self, v: &mut Vec<u8>, t: &Rc<ASTNode>, is_op: bool)
  -> Result<(),Error>
{
  let a = ast_argv(t);
  let mut jumps: Vec<usize> = Vec::new();
  let m = a.len()/2;
  for i in 0..m {
    try!(self.compile_ast(v,&a[2*i]));
    push_bc(v, bc::JZ, a[2*i].line, a[2*i].col);
    let index = v.len();
    push_u32(v,0xcafe);
    try!(self.compile_ast(v,&a[2*i+1]));
    push_bc(v, bc::JMP, t.line, t.col);
    jumps.push(v.len());
    push_u32(v,0xcafe);
    let len = v.len();
    write_i32(&mut v[index..index+4],(BCSIZE+len) as i32-index as i32);
  }
  if a.len()%2==1 {
    try!(self.compile_ast(v,&a[a.len()-1]));
  }else{
    if is_op {
      push_bc(v, bc::NULL, t.line, t.col);
    }
  }
  let len = v.len();
  for i in 0..m {
    let index = jumps[i];
    write_i32(&mut v[index..index+4],(BCSIZE+len) as i32-index as i32);
  }
  return Ok(());
}

fn compile_app(&mut self, v: &mut Vec<u8>, t: &Rc<ASTNode>)
  -> Result<(),Error>
{
  let a = ast_argv(t);

  // callee
  try!(self.compile_ast(v,&a[0]));

  // self argument
  push_bc(v, bc::NULL, t.line, t.col);

  // arguments
  for i in 1..a.len() {
    try!(self.compile_ast(v,&a[i]));
  }

  push_bc(v, bc::CALL, t.line, t.col);

  // argument count,
  // not counting the self argument,
  // not counting the callee
  push_u32(v, (a.len()-1) as u32);

  return Ok(());
}

fn compile_fn(&mut self, bv: &mut Vec<u8>, t: &Rc<ASTNode>)
  -> Result<(),Error>
{
  let a = ast_argv(t);
  let mut bv2: Vec<u8> = Vec::new();

  // A separator to identify a new code block. Just needed
  // to make the assembler listing human readable.
  push_bc(&mut bv2, bc::FNSEP, t.line, t.col);

  // Move self.fn_indices beside to allow nested functions.
  let fn_indices = replace(&mut self.fn_indices,Vec::new());

  // Every function has its own table of variables.
  let vtab = replace(&mut self.vtab,VarTab::new(t.s.clone()));
  self.arguments(&a[0]);

  // Compile the function body.
  self.function_nesting+=1;
  try!(self.compile_ast(&mut bv2,&a[1]));
  self.function_nesting-=1;
  
  let var_count = self.vtab.count_local;

  // Restore.
  self.vtab = vtab;

  // Shift the start adresses of nested functions
  // by the now known offset and turn them into
  // position independent code. The offset is negative
  // because the code blocks of nested functions come
  // before this code block. So we need to jump back.
  self.offsets(&mut bv2,-(self.bv_blocks.len() as i32));

  // Restore self.fn_indices.
  replace(&mut self.fn_indices,fn_indices);

  // Add an additional return statement that will be reached
  // in case the control flow reaches the end of the function.
  push_bc(&mut bv2, bc::RET, t.line, t.col);

  // The name of the function or null in case
  // the function is anonymous.
  push_bc(bv, bc::NULL, t.line, t.col);

  // Function constructor instruction.
  push_bc(bv, bc::FN, t.line, t.col);

  // Start address of the function body.
  // Add four to point behind FNSEP.
  // The size of bv will be added as the
  // compilation is finished.
  let index = bv.len();
  self.fn_indices.push(index);
  push_u32(bv,self.bv_blocks.len() as u32+4);

  let argv = ast_argv(&a[0]);
  push_i32(bv,argv.len() as i32); // minimal argument count
  push_i32(bv,argv.len() as i32); // maximal argument count
  push_u32(bv,var_count as u32); // number of local variables

  // Append the code block to the buffer of code blocks.
  self.bv_blocks.append(&mut bv2);

  return Ok(());
}

fn offsets(&self, bv: &mut Vec<u8>, offset: i32){
  for &index in &self.fn_indices {
    let x = compose_i32(&bv[index..index+4]);
    write_i32(&mut bv[index..index+4], x+BCSIZE as i32+offset-index as i32);
  }
}

fn string_literal(&mut self, s: &str) -> Result<String,Error> {
  let mut y = String::new();
  let mut escape = false;
  for c in s.chars() {
    if escape {
      if c=='n' {y.push('\n');}
      else if c=='s' {y.push(' ');}
      else if c=='t' {y.push('t');}
      else if c=='d' {y.push('"');}
      else if c=='q' {y.push('\'');}
      else if c=='b' {y.push('\\');}
      else if c=='r' {y.push('\r');}
      else if c=='e' {y.push('\x1b');}
      else if c=='f' {y.push('\x0c');}
      else if c=='0' {y.push('\x00');}
      else if c=='v' {y.push('\x0b');}
      else if c=='a' {y.push('\x07');}
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

fn arguments(&mut self, t: &ASTNode){
  let a = ast_argv(t);
  for i in 0..a.len() {
    if let Some(ref s) = a[i].s {
      self.vtab.v.push(VarInfo{
        s: s.clone(),
        index: self.vtab.count_arg,
        var_type: VarType::Argument
      });
      self.vtab.count_arg+=1;
    }
  }
}

fn index_type(&mut self, id: &str) -> Option<(usize,VarType)> {
  if let Some(ref s) = self.vtab.fn_id {
    if id==s {return Some((0,VarType::FnId));}
  }
  let a = &self.vtab.v[..];
  for i in 0..a.len() {
    if a[i].s==id {
      return Some((a[i].index, a[i].var_type));
    }
  }
  return None;
}

fn compile_variable(&mut self, bv: &mut Vec<u8>, t: &ASTNode)
  -> Result<(),Error>
{
  let key = match t.s {Some(ref x)=>x, None=>panic!()};
  match self.index_type(key) {
    Some((index,var_type)) => {
      match var_type {
        VarType::Argument => {
          push_bc(bv,bc::LOAD_ARG,t.line,t.col);
          push_u32(bv,index as u32+1);
        },
        VarType::Local => {
          push_bc(bv,bc::LOAD_LOCAL,t.line,t.col);
          push_u32(bv,index as u32);
        },
        VarType::FnId => {
          push_bc(bv,bc::FNSELF,t.line,t.col);
        }
        _ => {panic!()}
      }
      return Ok(());
    },
    None => {
      let index = self.get_index(key);
      push_bc(bv,bc::LOAD,t.line,t.col);
      push_u32(bv,index as u32);
      return Ok(());
    }
  };
}

fn compile_assignment(&mut self, bv: &mut Vec<u8>, t: &ASTNode,
  line: usize, col: usize
){
  let key = match t.s {Some(ref x)=>x, None=>panic!()};
  if self.function_nesting>0 {
    match self.index_type(key) {
      Some((index,var_type)) => {
        match var_type {
          VarType::Local => {
            push_bc(bv,bc::STORE_LOCAL,line,col);
            push_u32(bv,index as u32);
          },
          _ => panic!()
        }
      },
      None => {
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
  }else{
    let index = self.get_index(key);
    push_bc(bv,bc::STORE,line,col);
    push_u32(bv,index as u32);
  }
}

fn compile_ast(&mut self, bv: &mut Vec<u8>, t: &Rc<ASTNode>)
  -> Result<(),Error>
{
  if t.symbol_type == SymbolType::Identifier {
    try!(self.compile_variable(bv,t));
  }else if t.symbol_type == SymbolType::Operator {
    let value = t.value;
    if value == Symbol::Assignment {
      let a = ast_argv(t);
      try!(self.compile_ast(bv,&a[1]));
      if a[0].symbol_type == SymbolType::Identifier {
        self.compile_assignment(bv,&a[0],t.line,t.col);
      }else if a[0].value == Symbol::Index {
        let b = ast_argv(&a[0]);
        try!(self.compile_ast(bv,&b[0]));
        try!(self.compile_ast(bv,&b[1]));
        push_bc(bv,bc::SET_INDEX,t.line,t.col);
      }else{
        return Err(self.syntax_error(t.line,t.col,
          String::from("expected identifier before '='.")));
      }
    }else if value == Symbol::Plus {
      try!(self.compile_operator(bv,t,bc::ADD));
    }else if value == Symbol::Minus {
      try!(self.compile_operator(bv,t,bc::SUB));
    }else if value == Symbol::Ast {
      try!(self.compile_operator(bv,t,bc::MPY));
    }else if value == Symbol::Div {
      try!(self.compile_operator(bv,t,bc::DIV));
    }else if value == Symbol::Idiv {
      try!(self.compile_operator(bv,t,bc::IDIV));
    }else if value == Symbol::Neg {
      try!(self.compile_operator(bv,t,bc::NEG));
    }else if value == Symbol::Pow {
      try!(self.compile_operator(bv,t,bc::POW));
    }else if value == Symbol::Eq {
      try!(self.compile_operator(bv,t,bc::EQ));
    }else if value == Symbol::Ne {
      try!(self.compile_operator(bv,t,bc::NE));
    }else if value == Symbol::Lt {
      try!(self.compile_operator(bv,t,bc::LT));
    }else if value == Symbol::Gt {
      try!(self.compile_operator(bv,t,bc::GT));
    }else if value == Symbol::Le {
      try!(self.compile_operator(bv,t,bc::LE));
    }else if value == Symbol::Ge {
      try!(self.compile_operator(bv,t,bc::GE));
    }else if value == Symbol::Is {
      try!(self.compile_operator(bv,t,bc::IS));
    }else if value == Symbol::Isnot {
      try!(self.compile_operator(bv,t,bc::ISNOT));
    }else if value == Symbol::Index {
      try!(self.compile_operator(bv,t,bc::GET_INDEX));
    }else if value == Symbol::List {
      try!(self.compile_operator(bv,t,bc::LIST));
      let size = match t.a {Some(ref a) => a.len() as u32, None => panic!()};
      push_u32(bv,size);
    }else if value == Symbol::Map {
      try!(self.compile_operator(bv,t,bc::MAP));
      let size = match t.a {Some(ref a) => a.len() as u32, None => panic!()};
      push_u32(bv,size);
    }else if value == Symbol::Application {
      try!(self.compile_app(bv,t));
    }else if value == Symbol::If {
      try!(self.compile_if(bv,t,true));
    }else if value == Symbol::Dot {
      try!(self.compile_operator(bv,t,bc::DOT));
    }else{
      return Err(self.syntax_error(t.line,t.col,
        format!("cannot compile Operator '{}'.",symbol_to_string(t.value))
      ));
    }
  }else if t.symbol_type == SymbolType::Int {
    push_bc(bv, bc::INT, t.line, t.col);
    let x: i32 = match t.s {Some(ref x)=>x.parse().unwrap(), None=>panic!()};
    push_i32(bv,x);
  }else if t.symbol_type == SymbolType::Float {
    push_bc(bv, bc::FLOAT, t.line, t.col);
    let x: f64 = match t.s {Some(ref x)=>x.parse().unwrap(), None=>panic!()};
    push_u64(bv,unsafe{transmute::<f64,u64>(x)});
  }else if t.symbol_type == SymbolType::Imag {
    push_bc(bv, bc::IMAG, t.line, t.col);
    let x: f64 = match t.s {Some(ref x)=>x.parse().unwrap(), None=>panic!()};
    push_u64(bv,unsafe{transmute::<f64,u64>(x)});
  }else if t.value == Symbol::Null {
    push_bc(bv,bc::NULL,t.line,t.col);
  }else if t.value == Symbol::True {
    push_bc(bv,bc::TRUE,t.line,t.col);
  }else if t.value == Symbol::False {
    push_bc(bv,bc::FALSE,t.line,t.col);
  }else if t.symbol_type == SymbolType::Keyword {
    if t.value == Symbol::If {
      try!(self.compile_if(bv,t,false));
    }else if t.value == Symbol::While {
      try!(self.compile_while(bv,t));
    }else if t.value == Symbol::Block {
      let a = ast_argv(t);
      for i in 0..a.len() {
        try!(self.compile_ast(bv,&a[i]));
      }
    }else if t.value == Symbol::Sub {
      try!(self.compile_fn(bv,t));
    }else if t.value == Symbol::Statement {
      let a = ast_argv(t);
      try!(self.compile_ast(bv,&a[0]));
      push_bc(bv,bc::POP,t.line,t.col);
    }else{
      panic!();
    }
  }else if t.symbol_type == SymbolType::String {
    let s = match t.s {Some(ref x)=>x, None=>panic!()};
    let key = try!(self.string_literal(s));
    let index = self.get_index(&key);
    push_bc(bv,bc::STR,t.line,t.col);
    push_u32(bv,index as u32);
  }else{
    panic!();
  }
  return Ok(());
}

}//impl Compilation

fn ast_argv(t: &ASTNode) -> &Box<[Rc<ASTNode>]>{
  match t.a {Some(ref x)=> x, None=>panic!()}
}

fn push_u16(v: &mut Vec<u8>, x: u16){
  v.push(x as u8);
  v.push((x>>8) as u8);
}

fn push_u32(v: &mut Vec<u8>, x: u32){
  v.push(x as u8);
  v.push((x>>8) as u8);
  v.push((x>>16) as u8);
  v.push((x>>24) as u8);
}

fn push_i32(v: &mut Vec<u8>, x: i32){
  let x = unsafe{transmute::<i32,u32>(x)};
  v.push(x as u8);
  v.push((x>>8) as u8);
  v.push((x>>16) as u8);
  v.push((x>>24) as u8);
}

fn write_i32(a: &mut [u8], x: i32){
  let x = unsafe{transmute::<i32,u32>(x)};
  a[0] = x as u8;
  a[1] = (x>>8) as u8;
  a[2] = (x>>16) as u8;
  a[3] = (x>>24) as u8;
}

fn push_u64(v: &mut Vec<u8>, x: u64){
  v.push(x as u8);
  v.push((x>>8) as u8);
  v.push((x>>16) as u8);
  v.push((x>>24) as u8);
  v.push((x>>32) as u8);
  v.push((x>>40) as u8);
  v.push((x>>48) as u8);
  v.push((x>>56) as u8);
}

fn push_line_col(v: &mut Vec<u8>, line: usize, col: usize){
  push_u16(v,line as u16);
  v.push(col as u8);
}

fn push_bc(v: &mut Vec<u8>, byte: u8, line: usize, col: usize){
  v.push(byte);
  push_line_col(v,line,col);
}

fn compose_u16(b0: u8, b1: u8) -> u16 {
  (b1 as u16)<<8 | (b0 as u16)
}

fn compose_i32(a: &[u8]) -> i32 {
  (a[3] as i32)<<24 | (a[2] as i32)<<16 | (a[1] as i32)<<8 | (a[0] as i32)
}

fn compose_u32(a: &[u8]) -> u32 {
  (a[3] as u32)<<24 | (a[2] as u32)<<16 | (a[1] as u32)<<8 | (a[0] as u32)
}

fn compose_u64(a: &[u8]) -> u64 {
   (a[7] as u64)<<56 | (a[6] as u64)<<48 | (a[5] as u64)<<40 | (a[4] as u64)<<32
 | (a[3] as u64)<<24 | (a[2] as u64)<<16 | (a[1] as u64)<< 8 | (a[0] as u64)
}

fn asm_listing(a: &[u8]) -> String {
  let mut s = String::from("Adr | Line:Col| Operation\n");
  let mut i=0;
  while i<a.len() {
    let op = a[i];
    if op != bc::FNSEP {
      let u = format!("{:04}| {:4}:{:02} | ",i,compose_u16(a[i+1],a[i+2]),a[i+3]);
      s.push_str(&u);
    }
    match op {
      bc::INT => {
        let x = compose_i32(&a[BCSIZE+i..BCSIZE+i+4]);
        let u = format!("push int {} (0x{:x})\n",x,x);
        s.push_str(&u);
        i+=BCSIZE+4;
      },
      bc::FLOAT => {
        let x = unsafe{transmute::<u64,f64>(
          compose_u64(&a[BCSIZE+i..BCSIZE+i+8])
        )};
        let u = format!("push float {}\n",x);
        s.push_str(&u);
        i+=BCSIZE+8;
      },
      bc::IMAG => {
        let x = unsafe{transmute::<u64,f64>(
          compose_u64(&a[BCSIZE+i..BCSIZE+i+8])
        )};
        let u = format!("push imag {}\n",x);
        s.push_str(&u);
        i+=BCSIZE+8;
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
      bc::POW => {s.push_str("pow\n"); i+=BCSIZE;},
      bc::NEG => {s.push_str("neg\n"); i+=BCSIZE;},
      bc::EQ => {s.push_str("eq\n"); i+=BCSIZE;},
      bc::NE => {s.push_str("not eq\n"); i+=BCSIZE;},
      bc::LT => {s.push_str("lt\n"); i+=BCSIZE;},
      bc::GT => {s.push_str("gt\n"); i+=BCSIZE;},
      bc::LE => {s.push_str("le\n"); i+=BCSIZE;},
      bc::GE => {s.push_str("not ge\n"); i+=BCSIZE;},
      bc::IS => {s.push_str("is\n"); i+=BCSIZE;},
      bc::ISNOT => {s.push_str("is not\n"); i+=BCSIZE;},
      bc::IN => {s.push_str("in\n"); i+=BCSIZE;},
      bc::NOTIN => {s.push_str("not in\n"); i+=BCSIZE;},
      bc::LIST => {
        let x = compose_u32(&a[BCSIZE+i..BCSIZE+i+4]);
        let u = format!("list, size={}\n",x);
        s.push_str(&u);
        i+=BCSIZE+4;
      },
      bc::MAP => {
        let x = compose_u32(&a[BCSIZE+i..BCSIZE+i+4]);
        let u = format!("map, size={}\n",x);
        s.push_str(&u);
        i+=BCSIZE+4;
      },
      bc::LOAD => {
        let x = compose_u32(&a[BCSIZE+i..BCSIZE+i+4]);
        let u = format!("load global [{}]\n",x);
        s.push_str(&u);
        i+=BCSIZE+4;
      },
      bc::LOAD_ARG => {
        let x = compose_u32(&a[BCSIZE+i..BCSIZE+i+4]);
        let u = format!("load argument [{}]\n",x);
        s.push_str(&u);
        i+=BCSIZE+4;
      },
      bc::LOAD_LOCAL => {
        let x = compose_u32(&a[BCSIZE+i..BCSIZE+i+4]);
        let u = format!("load local [{}]\n",x);
        s.push_str(&u);
        i+=BCSIZE+4;
      },
      bc::LOAD_CONTEXT => {
        let x = compose_u32(&a[BCSIZE+i..BCSIZE+i+4]);
        let u = format!("load context [{}]\n",x);
        s.push_str(&u);
        i+=BCSIZE+4;
      },
      bc::STORE => {
        let x = compose_u32(&a[BCSIZE+i..BCSIZE+i+4]);
        let u = format!("store global [{}]\n",x);
        s.push_str(&u);
        i+=BCSIZE+4;
      },
      bc::STORE_ARG => {
        let x = compose_u32(&a[BCSIZE+i..BCSIZE+i+4]);
        let u = format!("store argument [{}]\n",x);
        s.push_str(&u);
        i+=BCSIZE+4;
      },
      bc::STORE_LOCAL => {
        let x = compose_u32(&a[BCSIZE+i..BCSIZE+i+4]);
        let u = format!("store local [{}]\n",x);
        s.push_str(&u);
        i+=BCSIZE+4;
      },
      bc::STORE_CONTEXT => {
        let x = compose_u32(&a[BCSIZE+i..BCSIZE+i+4]);
        let u = format!("store context [{}]\n",x);
        s.push_str(&u);
        i+=BCSIZE+4;
      },
      bc::STR => {
        let x = compose_u32(&a[BCSIZE+i..BCSIZE+i+4]);
        let u = format!("str literal [{}]\n",x);
        s.push_str(&u);
        i+=BCSIZE+4;
      },
      bc::JMP => {
        let x = compose_i32(&a[BCSIZE+i..BCSIZE+i+4]);

        // Resolve position independent code
        // to make the listing human readable.
        let u = format!("jmp {}\n",i as i32+x);
        s.push_str(&u);
        i+=BCSIZE+4;
      },
      bc::JZ => {
        let x = compose_i32(&a[BCSIZE+i..BCSIZE+i+4]);
        let u = format!("jz {}\n",i as i32+x);
        s.push_str(&u);
        i+=BCSIZE+4;
      },
      bc::JNZ => {
        let x = compose_i32(&a[BCSIZE+i..BCSIZE+i+4]);
        let u = format!("jnz {}\n",i as i32+x);
        s.push_str(&u);
        i+=BCSIZE+4;
      },
      bc::CALL => {
        let argc = compose_u32(&a[BCSIZE+i..BCSIZE+i+4]);
        let u = format!("call, argc={}\n",argc);
        s.push_str(&u);
        i+=BCSIZE+4;
      },
      bc::RET => {s.push_str("ret\n"); i+=BCSIZE;},
      bc::FNSEP => {s.push_str("\nFunction\n"); i+=BCSIZE;},
      bc::FN => {
        let address = compose_i32(&a[BCSIZE+i..BCSIZE+i+4]);
        let argc_min = compose_i32(&a[BCSIZE+i+4..BCSIZE+i+8]);
        let argc_max = compose_i32(&a[BCSIZE+i+8..BCSIZE+i+12]);
        let var_count = compose_i32(&a[BCSIZE+i+12..BCSIZE+i+16]);

        // Resolve position independent code
        // to make the listing human readable.
        let u = format!("fn [{}], argc_min={}, argc_max={}\n",i as i32+address,argc_min,argc_max);
        s.push_str(&u);
        i+=BCSIZE+16;
      },
      bc::GET_INDEX => {s.push_str("get index\n"); i+=BCSIZE;},
      bc::SET_INDEX => {s.push_str("set index\n"); i+=BCSIZE;},
      bc::DOT => {s.push_str("dot\n"); i+=BCSIZE;},
      bc::DOT_SET => {s.push_str("dot set\n"); i+=BCSIZE;},
      bc::SWAP => {s.push_str("swap\n"); i+=BCSIZE;},
      bc::DUP => {s.push_str("dup\n"); i+=BCSIZE;},
      bc::POP => {s.push_str("pop\n"); i+=BCSIZE;},
      bc::HALT => {s.push_str("halt\n"); i+=BCSIZE;},
      _ => {unimplemented!();}
    }
  }
  return s;
}

fn print_asm_listing(a: &[u8]){
  let s = asm_listing(a);
  println!("{}",&s);
}

fn print_data(a: &[Object]){
  println!("Data");
  for i in 0..a.len() {
    println!("[{}]: {}",i,::vm::object_to_repr(&a[i]));
  }
  if a.len()==0 {
    println!("empty");
  }else{
    println!();
  }
}

pub fn compile(v: Vec<Token>, mode_cmd: bool,
  history: &mut system::History, id: &str
) -> Result<Rc<Module>,Error>
{
  let mut compilation = Compilation{
    mode_cmd: mode_cmd, index: 0,
    parens: false, statement: !mode_cmd,
    history: history, file: id, stab: HashMap::new(),
    stab_index: 0, data: Vec::new(), bv_blocks: Vec::new(),
    fn_indices: Vec::new(), vtab: VarTab::new(None),
    function_nesting: 0
  };
  let mut i = TokenIterator{index: 0, a: Rc::new(v.into_boxed_slice())};
  let y = try!(compilation.ast(&mut i));
  print_ast(&y,2);

  let mut bv: Vec<u8> = Vec::new();
  try!(compilation.compile_ast(&mut bv, &y));
  push_bc(&mut bv, bc::HALT, y.line, y.col);
  let len = bv.len();
  compilation.offsets(&mut bv, len as i32);

  bv.append(&mut compilation.bv_blocks);

  print_asm_listing(&bv);
  print_data(&compilation.data);
  // let m = Rc::new(Module{program: bv, data: compilation.data});
  let m = Rc::new(Module{program: Rc::new(bv), data: compilation.data});
  return Ok(m);
}
