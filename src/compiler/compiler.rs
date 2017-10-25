
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use system;
use std::ascii::AsciiExt;
use std::rc::Rc;
use vm::bc;
use vm::BCSIZE;

#[derive(Copy,Clone,PartialEq)]
enum SymbolType{
  Operator, Separator, Bracket, Bool, Int,
  String, Identifier, Keyword
}

#[derive(Copy,Clone,PartialEq)]
enum Symbol{
  None, Plus, Minus, Ast, Div, Idiv, Mod, Pow,
  Lt, Gt, Le, Ge, Eq, Ne, In, Is, Isin, Notin, Range,
  And, Or, Amp, Vline, Neg, Not, Tilde, Svert, Assignment,
  PLeft, PRight, BLeft, BRight, CLeft, CRight, Newline,
  Lshift, Rshift, Assert, Begin, Break, Catch, Continue,
  Elif, Else, End, For, Global, Goto, Label,
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

pub fn print_syntax_error(e: SyntaxError){
  println!("Line {}, col {} ({}):",e.line,e.col,e.file);
  println!("Syntax error: {}",e.s);
}

fn compiler_error() -> !{
  panic!("compiler error");
}

fn is_keyword(id: &String) -> Option<&'static KeywordsElement> {
  // let mut i: usize;
  let n: usize = KEYWORDS.len();
  // i=0;
  for i in 0..n {
    if KEYWORDS[i].s==id  {return Some(&KEYWORDS[i]);}
  }
  return None;
}

pub fn scan(s: &str, line_start: usize, file: &str) -> Result<Vec<Token>, SyntaxError>{
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
      while i<n && a[i].is_digit(10) {
        i+=1; col+=1;
      }
      let number: &String = &a[j..i].iter().cloned().collect();
      v.push(Token{token_type: SymbolType::Int,
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
        '!' => {
          if i+1<n && a[i+1]=='=' {
            v.push(Token{token_type: SymbolType::Operator,
              value: Symbol::Ne, line: line, col: col, s: None});
            i+=2; col+=2;
          }else{
            return Err(SyntaxError{line: line, col: col, file: String::from(file),
              s: format!("unexpected character '{}'.", c)});          
          }
        },
        '#' => {
          while i<n && a[i]!='\n' {i+=1; col+=1;}
          v.push(Token{token_type: SymbolType::Separator,
            value: Symbol::Newline, line: line, col: col, s: None});
          i+=1; col=1; line+=1;
        },
        _ => {
          return Err(SyntaxError{line: line, col: col, file: String::from(file),
            s: format!("unexpected character '{}'.", c)});
        }
      }
    }
  }
  v.push(Token{token_type: SymbolType::Separator,
    value: Symbol::Terminal, line: line, col: col, s: None});
  return Ok(v);
}

fn token_value_to_string(value: Symbol) -> &'static str {
  return match value {
    Symbol::Plus => "+",  Symbol::Minus => "-",
    Symbol::Ast  => "*",  Symbol::Div => "/",
    Symbol::Mod  => "%",  Symbol::Pow => "^",
    Symbol::Vline=> "|",  Symbol::Amp => "&",
    Symbol::Idiv => "//", Symbol::Svert=> "$",
    Symbol::In   => "in", Symbol::Is => "is",
    Symbol::Isin=>"is in",Symbol::Notin=> "not in",
    Symbol::And  => "and",Symbol::Or => "or",
    Symbol::Not  => "not",Symbol::Tilde => "~",
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
    SymbolType::String | SymbolType::Int | SymbolType::Identifier => {
      print!("[{}]",match x.s {Some(ref s) => s, None => compiler_error()});
    },
    SymbolType::Operator | SymbolType::Separator |
    SymbolType::Bracket  | SymbolType::Keyword | SymbolType::Bool => {
      print!("[{}]",token_value_to_string(x.value));
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
    SymbolType::Identifier | SymbolType::Int => {
      println!("{}",match t.s {Some(ref s) => s, None => compiler_error()});
    },
    SymbolType::String => {
      println!("\"{}\"",match t.s {Some(ref s) => s, None => compiler_error()});    
    },
    SymbolType::Operator | SymbolType::Separator |
    SymbolType::Keyword  | SymbolType::Bool => {
      println!("{}",token_value_to_string(t.value));
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

fn scan_line(line_start: usize, h: &mut system::History) -> Result<Vec<Token>,SyntaxError>{
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

pub struct Compilation<'a>{
  mode_cmd: bool,
  index: usize,
  parens: bool,
  history: &'a mut system::History,
  file: &'a str
}

struct TokenIterator{
  pub a: Rc<Box<[Token]>>,
  pub index: usize
}

impl TokenIterator{
  fn next_token(&mut self, c: &mut Compilation) -> Result<Rc<Box<[Token]>>,SyntaxError>{
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
  fn next_any_token(&mut self, c: &mut Compilation) -> Result<Rc<Box<[Token]>>,SyntaxError>{
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

fn syntax_error(&self, line: usize, col: usize, s: String) -> SyntaxError{
  SyntaxError{line: line, col: col,
    file: String::from(self.file), s: s
  }
}

fn unexpected_token(&mut self, line: usize, col: usize, value: Symbol) -> SyntaxError{
  // panic!();
  SyntaxError{line: line, col: col, file: String::from(self.file),
    s: format!("unexpected token: '{}'.",token_value_to_string(value))
  }
}

fn list_literal(&mut self, i: &mut TokenIterator) -> Result<Box<[Rc<ASTNode>]>,SyntaxError> {
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

fn map_literal(&mut self, i: &mut TokenIterator) -> Result<Box<[Rc<ASTNode>]>,SyntaxError> {
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

fn table_literal(&mut self, i: &mut TokenIterator, t0: &Token) -> Result<Rc<ASTNode>,SyntaxError> {
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

fn application(&mut self, i: &mut TokenIterator, f: Rc<ASTNode>, terminal: Symbol)
  -> Result<Rc<ASTNode>,SyntaxError>
{
  let mut v: Vec<Rc<ASTNode>> = Vec::new();
  let line = f.line;
  let col = f.col;
  v.push(f);
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
  let value = if terminal==Symbol::PRight
    {Symbol::Application}
  else
    {Symbol::Index};
  return Ok(Rc::new(ASTNode{line: line, col: col, symbol_type: SymbolType::Operator,
    value: value, info: Info::None, s: None, a: Some(v.into_boxed_slice())}));
}

fn atom(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError> {
  let p = try!(i.next_token(self));
  let t = &p[i.index];
  let y;
  if t.token_type==SymbolType::Identifier || t.token_type==SymbolType::Int ||
     t.token_type==SymbolType::String
  {
    i.index+=1;
    y = Rc::new(ASTNode{line: t.line, col: t.col, symbol_type: t.token_type,
      value: Symbol::Null, info: Info::None, s: t.s.clone(), a: None});
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
  }else{
    return Err(self.unexpected_token(t.line, t.col, t.value));
  }
  let p2 = try!(i.next_any_token(self));
  let t2 = &p2[i.index];
  if t2.value == Symbol::PLeft {
    i.index+=1;
    return self.application(i,y,Symbol::PRight);
  }else if t2.value == Symbol::BLeft {
    i.index+=1;
    return self.application(i,y,Symbol::BRight);
  }else{
    return Ok(y);
  }
}

fn power(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
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

fn signed_expression(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
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

fn factor(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
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

fn addition(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
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

fn shift(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
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

fn intersection(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
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

fn union(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
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

fn range(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
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

fn comparison(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
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

fn eq_expression(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
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

fn negation(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
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

fn conjunction(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
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

fn disjunction(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
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

fn if_expression(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
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

fn expression(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
  return self.if_expression(i);
}

fn assignment(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
  let x = try!(self.expression(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value==Symbol::Assignment {
    i.index+=1;
    let y = try!(self.expression(i));
    return Ok(binary_operator(t.line,t.col,Symbol::Assignment,x,y));
  }else if self.mode_cmd {
    return Ok(x);
  }else{
    return Ok(Rc::new(ASTNode{line: t.line, col: t.col, symbol_type: SymbolType::Keyword,
      value: Symbol::Statement, info: Info::None, s: None, a: Some(Box::new([x]))}));
  }
}

fn while_statement(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
  let condition = try!(self.expression(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value == Symbol::Do || t.value == Symbol::Newline {
    i.index+=1;
  }else{
    return Err(self.syntax_error(t.line, t.col, String::from("expected 'do' or a line break.")));
  }
  let body = try!(self.statements(i));
  return Ok(Rc::new(ASTNode{line: t.line, col: t.col, symbol_type: SymbolType::Keyword,
    value: Symbol::While, info: Info::None, s: None, a: Some(Box::new([condition,body]))}));  
}

fn for_statement(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
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
  let body = try!(self.statements(i));
  return Ok(Rc::new(ASTNode{line: t.line, col: t.col, symbol_type: SymbolType::Keyword,
    value: Symbol::For, info: Info::None, s: None, a: Some(Box::new([variable,a,body]))}));  
}

fn if_statement(&mut self, i: &mut TokenIterator, t0: &Token) -> Result<Rc<ASTNode>,SyntaxError>{
  let mut v: Vec<Rc<ASTNode>> = Vec::new();
  let condition = try!(self.expression(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value == Symbol::Then || t.value == Symbol::Newline {
    i.index+=1;
  }else{
    return Err(self.syntax_error(t.line, t.col, String::from("expected 'then' or a line break.")));
  }
  let body = try!(self.statements(i));
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
      let body = try!(self.statements(i));
      v.push(condition);
      v.push(body);
    }else if t.value == Symbol::Else {
      i.index+=1;
      let body = try!(self.statements(i));
      v.push(body);
    }else if t.value == Symbol::End {
      break;
    }
  }
  return Ok(Rc::new(ASTNode{line: t0.line, col: t0.col, symbol_type: SymbolType::Keyword,
    value: Symbol::If, info: Info::None, s: None, a: Some(v.into_boxed_slice())}));
}

fn statements(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
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
      let x = try!(self.while_statement(i));
      v.push(x);
      let p = try!(i.next_any_token(self));
      let t = &p[i.index];
      if t.value != Symbol::End {    
        return Err(self.syntax_error(t.line, t.col, String::from("expected 'end'.")));
      }
      i.index+=1;
    }else if t.value == Symbol::For {
      i.index+=1;
      let x = try!(self.for_statement(i));
      v.push(x);
      let p = try!(i.next_any_token(self));
      let t = &p[i.index];
      if t.value != Symbol::End {
        return Err(self.syntax_error(t.line, t.col, String::from("expected 'end'.")));    
      }
      i.index+=1;
    }else if t.value == Symbol::If {
      i.index+=1;
      let x = try!(self.if_statement(i,t));
      v.push(x);
      let p = try!(i.next_any_token(self));
      let t = &p[i.index];
      if t.value != Symbol::End {
        return Err(self.syntax_error(t.line, t.col, String::from("expected 'end'.")));
      }
      i.index+=1;
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
  if v.len()==1 {
    return Ok(match v.pop() {Some(x) => x, None => compiler_error()});
  }else{
    return Ok(Rc::new(ASTNode{line: t0.line, col: t0.col, symbol_type: SymbolType::Keyword,
      value: Symbol::Block, info: Info::None, s: None, a: Some(v.into_boxed_slice())}));
  }
}

fn ast(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
  return self.statements(i);
}

fn compile_operator(&mut self, v: &mut Vec<u8>, t: &Rc<ASTNode>, byte_code: u8) -> Result<(),SyntaxError>{
  let a = ast_argv(t);
  for i in 0..a.len() {
    try!(self.compile_ast(v,&a[i]));
  }
  v.push(byte_code);
  push_line_col(v,t.line,t.col);
  return Ok(());
}

fn compile_ast(&mut self, v: &mut Vec<u8>, t: &Rc<ASTNode>) -> Result<(),SyntaxError>{
  if t.symbol_type == SymbolType::Operator {
    let value = t.value;
    if value == Symbol::Plus {
      try!(self.compile_operator(v,t,bc::ADD));
    }else if value == Symbol::Minus {
      try!(self.compile_operator(v,t,bc::SUB));
    }else if value == Symbol::Ast {
      try!(self.compile_operator(v,t,bc::MPY));
    }else if value == Symbol::Div {
      try!(self.compile_operator(v,t,bc::DIV));
    }else if value == Symbol::Idiv {
      try!(self.compile_operator(v,t,bc::IDIV));
    }else if value == Symbol::Neg {
      try!(self.compile_operator(v,t,bc::NEG));
    }else if value == Symbol::List {
      try!(self.compile_operator(v,t,bc::LIST));
      let size = match t.a {Some(ref a) => a.len() as i32, None => panic!()};
      push_i32(v,size);
    }else if value == Symbol::Map {
      try!(self.compile_operator(v,t,bc::MAP));
      let size = match t.a {Some(ref a) => a.len() as i32, None => panic!()};
      push_i32(v,size);      
    }else{
      return Err(self.syntax_error(t.line,t.col,
        format!("cannot compile Operator '{}'.",token_value_to_string(t.value))
      ));
    }
  }else if t.symbol_type == SymbolType::Int {
    v.push(bc::INT);
    push_line_col(v,t.line,t.col);
    let x: i32 = match t.s {Some(ref x)=>x.parse().unwrap(), None=>panic!()};
    push_i32(v,x);
  }else if t.value == Symbol::Null {
    v.push(bc::NULL);
    push_line_col(v,t.line,t.col);
  }
  return Ok(());
}

}//impl Compilation

fn ast_argv(t: &Rc<ASTNode>) -> &Box<[Rc<ASTNode>]>{
  match t.a {Some(ref x)=> x, None=>panic!()}
}

fn push_i32(v: &mut Vec<u8>, x: i32){
  let x = x as u32;
  v.push(x as u8);
  v.push((x>>8) as u8);
  v.push((x>>16) as u8);
  v.push((x>>24) as u8);
}

fn push_u16(v: &mut Vec<u8>, x: u16){
  v.push(x as u8);
  v.push((x>>8) as u8);
}

fn push_line_col(v: &mut Vec<u8>, line: usize, col: usize){
  push_u16(v,line as u16);
  v.push(col as u8);
}

fn compose_u16(b1: u8, b2: u8) -> u16{
  return  (b2 as u16)<<8 | b1 as u16;
}

fn compose_i32(b1: u8, b2: u8, b3: u8, b4: u8) -> i32{
  return (b4 as i32)<<24 | (b3 as i32)<<16 | (b2 as i32)<<8 | (b1 as i32);
}

fn asm_listing(a: &[u8]) -> String{
  let mut s = String::from("Addr| Line:Col| Operation\n");
  let mut i=0;
  while i<a.len() {
    let u = format!("{:04}| {:4}:{:02} | ",i,compose_u16(a[i+1],a[i+2]),a[i+3]);
    s.push_str(&u);
    let byte = a[i];
    if byte==bc::INT {
      let x = compose_i32(a[BCSIZE+i],a[BCSIZE+i+1],a[BCSIZE+i+2],a[BCSIZE+i+3]);
      let u = format!("push int {} (0x{:x})\n",x,x);
      s.push_str(&u);
      i+=BCSIZE+4;
    }else if byte==bc::NULL {
      s.push_str("null\n");
      i+=BCSIZE;
    }else if byte==bc::ADD {
      s.push_str("add\n");
      i+=BCSIZE;
    }else if byte==bc::SUB {
      s.push_str("sub\n");
      i+=BCSIZE;
    }else if byte==bc::MPY {
      s.push_str("mpy\n");
      i+=BCSIZE;
    }else if byte==bc::DIV {
      s.push_str("div\n");
      i+=BCSIZE;
    }else if byte==bc::IDIV {
      s.push_str("idiv\n");
      i+=BCSIZE;
    }else if byte==bc::NEG {
      s.push_str("neg\n");
      i+=BCSIZE;
    }else if byte==bc::LIST {
      s.push_str("list\n");
      i+=BCSIZE+4;
    }else if byte==bc::MAP {
      s.push_str("map\n");
      i+=BCSIZE+4;
    }else if byte==bc::HALT {
      s.push_str("halt\n");
      break;
    }else{
      unimplemented!();
    }
  }
  return s;
}

fn print_asm_listing(a: &[u8]){
  let s = asm_listing(a);
  println!("{}",&s);
}

pub fn compile(v: Vec<Token>, mode_cmd: bool, history: &mut system::History, id: &str) -> Result<Vec<u8>,SyntaxError>{
  let mut compilation = Compilation{
    mode_cmd: mode_cmd, index: 0, parens: false,
    history: history, file: id
  };
  let mut i = TokenIterator{index: 0, a: Rc::new(v.into_boxed_slice())};
  let y = try!(compilation.ast(&mut i));
  print_ast(&y,2);
  let mut v: Vec<u8> = Vec::new();
  compilation.compile_ast(&mut v, &y).ok();
  v.push(bc::HALT as u8);
  push_line_col(&mut v,y.line,y.col);

  print_asm_listing(&v);
  return Ok(v);
}
