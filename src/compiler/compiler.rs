
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use system;
use std::ascii::AsciiExt;
use std::rc::Rc;

#[derive(Copy,Clone,PartialEq)]
enum TokenType{
  Terminal, Operator, Separator, Bracket, Bool, Int,
  String, Identifier, Keyword
}

#[derive(Copy,Clone,PartialEq)]
enum TokenValue{
  None, Plus, Minus, Ast, Div, Idiv, Mod, Pow,
  Lt, Gt, Le, Ge, Eq, Ne, In, Is,
  And, Or, Amp, Vline, Not, Tilde, Svert, Assignment,
  PLeft, PRight, BLeft, BRight, CLeft, CRight, Newline,
  Assert, Begin, Break, Catch, Continue,
  Elif, Else, End, For, Global, Goto, Label,
  If, While, Do, Raise, Return, Sub, Table, Then, Try,
  Use, Yield, True, False, Null, Dot, Comma, Colon, Semicolon,
  List, Map, Application, Index
}

pub struct Token {
  token_type: TokenType,
  value: TokenValue,
  line: usize,
  col: usize,
  s: Option<String>
}

struct KeywordsElement {
  s: &'static str,
  t: &'static TokenType,
  v: &'static TokenValue
}

static KEYWORDS: &'static [KeywordsElement] = &[
   KeywordsElement{s: "assert",  t: &TokenType::Keyword, v: &TokenValue::Assert},
   KeywordsElement{s: "and",     t: &TokenType::Operator,v: &TokenValue::And},
   KeywordsElement{s: "begin",   t: &TokenType::Keyword, v: &TokenValue::Begin},
   KeywordsElement{s: "break",   t: &TokenType::Keyword, v: &TokenValue::Begin},
   KeywordsElement{s: "catch",   t: &TokenType::Keyword, v: &TokenValue::Catch},
   KeywordsElement{s: "continue",t: &TokenType::Keyword, v: &TokenValue::Continue},
   KeywordsElement{s: "do",      t: &TokenType::Keyword, v: &TokenValue::Do},
   KeywordsElement{s: "elif",    t: &TokenType::Keyword, v: &TokenValue::Elif},
   KeywordsElement{s: "else",    t: &TokenType::Keyword, v: &TokenValue::Else},
   KeywordsElement{s: "end",     t: &TokenType::Keyword, v: &TokenValue::End},
   KeywordsElement{s: "false",   t: &TokenType::Bool,    v: &TokenValue::False},
   KeywordsElement{s: "for",     t: &TokenType::Keyword, v: &TokenValue::For},
   KeywordsElement{s: "global",  t: &TokenType::Keyword, v: &TokenValue::Global},
   KeywordsElement{s: "goto",    t: &TokenType::Keyword, v: &TokenValue::Goto},
   KeywordsElement{s: "label",   t: &TokenType::Keyword, v: &TokenValue::Label},
   KeywordsElement{s: "if",      t: &TokenType::Keyword, v: &TokenValue::If},
   KeywordsElement{s: "in",      t: &TokenType::Operator,v: &TokenValue::In},
   KeywordsElement{s: "is",      t: &TokenType::Operator,v: &TokenValue::Is},
   KeywordsElement{s: "not",     t: &TokenType::Operator,v: &TokenValue::Not},
   KeywordsElement{s: "null",    t: &TokenType::Keyword, v: &TokenValue::Null},
   KeywordsElement{s: "or",      t: &TokenType::Operator,v: &TokenValue::Or},
   KeywordsElement{s: "raise",   t: &TokenType::Keyword, v: &TokenValue::Raise},
   KeywordsElement{s: "return",  t: &TokenType::Keyword, v: &TokenValue::Return},
   KeywordsElement{s: "sub",     t: &TokenType::Keyword, v: &TokenValue::Sub},
   KeywordsElement{s: "table",   t: &TokenType::Keyword, v: &TokenValue::Table},
   KeywordsElement{s: "then",    t: &TokenType::Keyword, v: &TokenValue::Then},
   KeywordsElement{s: "true",    t: &TokenType::Bool,    v: &TokenValue::True},
   KeywordsElement{s: "try",     t: &TokenType::Keyword, v: &TokenValue::Try},
   KeywordsElement{s: "use",     t: &TokenType::Keyword, v: &TokenValue::Use},
   KeywordsElement{s: "while",   t: &TokenType::Keyword, v: &TokenValue::While},
   KeywordsElement{s: "yield",   t: &TokenType::Keyword, v: &TokenValue::Yield}
];

pub struct SyntaxError {
  line: usize, col: usize,
  s: String
}

pub fn print_syntax_error(e: SyntaxError){
  println!("Line {}, col {}:",e.line,e.col);
  println!("Syntax error: {}",e.s);
}

fn compiler_error() -> !{
  panic!("compiler error");
}

fn is_keyword(id: &String) -> Option<&'static KeywordsElement> {
  let mut i: usize;
  let n: usize = KEYWORDS.len();
  i=0;
  while i<n {
    if KEYWORDS[i].s==id  {return Some(&KEYWORDS[i]);}
    i+=1;
  }
  return None;
}

pub fn scan(s: &String, line_start: usize) -> Result<Vec<Token>, SyntaxError>{
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
      v.push(Token{token_type: TokenType::Int,
        value: TokenValue::None, line: line, col: hcol, s: Some(number.clone())});
    }else if (c.is_alphabetic() && c.is_ascii()) || a[i]=='_' {
      let j=i; hcol=col;
      while i<n && (a[i].is_alphabetic() || a[i].is_digit(10) || a[i]=='_') {
        i+=1; col+=1;
      }
      let id: &String = &a[j..i].iter().cloned().collect();
      match is_keyword(id) {
        Some(x) => {
          v.push(Token{token_type: *x.t, value: *x.v,
            line: line, col: hcol, s: None});
        },
        None => {
          v.push(Token{token_type: TokenType::Identifier,
            value: TokenValue::None, line: line, col: hcol, s: Some(id.clone())});
        }
      }
    }else{
      match c {
        ' ' | '\t' => {
          i+=1; col+=1;
        },
        '\n' => {
          v.push(Token{token_type: TokenType::Separator,
            value: TokenValue::Newline, line: line, col: col, s: None});
          i+=1; col=1; line+=1;
        },
        ',' => {
          v.push(Token{token_type: TokenType::Separator,
            value: TokenValue::Comma, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        ':' => {
          v.push(Token{token_type: TokenType::Separator,
            value: TokenValue::Colon, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        ';' => {
          v.push(Token{token_type: TokenType::Separator,
            value: TokenValue::Semicolon, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '(' => {
          v.push(Token{token_type: TokenType::Bracket,
            value: TokenValue::PLeft, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        ')' => {
          v.push(Token{token_type: TokenType::Bracket,
            value: TokenValue::PRight, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '[' => {
          v.push(Token{token_type: TokenType::Bracket,
            value: TokenValue::BLeft, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        ']' => {
          v.push(Token{token_type: TokenType::Bracket,
            value: TokenValue::BRight, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '{' => {
          v.push(Token{token_type: TokenType::Bracket,
            value: TokenValue::CLeft, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '}' => {
          v.push(Token{token_type: TokenType::Bracket,
            value: TokenValue::CRight, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '=' => {
          if i+1<n && a[i+1]=='=' {
            v.push(Token{token_type: TokenType::Operator,
              value: TokenValue::Eq, line: line, col: col, s: None});
            i+=2; col+=2;
          }else{
            v.push(Token{token_type: TokenType::Operator,
              value: TokenValue::Assignment, line: line, col: col, s: None});
            i+=1; col+=1;
          }
        },
        '+' => {
          v.push(Token{token_type: TokenType::Operator,
            value: TokenValue::Plus, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '-' => {
          v.push(Token{token_type: TokenType::Operator,
            value: TokenValue::Minus, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '*' => {
          v.push(Token{token_type: TokenType::Operator,
            value: TokenValue::Ast, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '/' => {
          if i+1<n && a[i+1]=='/' {
            v.push(Token{token_type: TokenType::Operator,
              value: TokenValue::Idiv, line: line, col: col, s: None});
            i+=2; col+=2;
          }else{
            v.push(Token{token_type: TokenType::Operator,
              value: TokenValue::Div, line: line, col: col, s: None});
            i+=1; col+=1;
          }
        },
        '%' => {
          v.push(Token{token_type: TokenType::Operator,
            value: TokenValue::Mod, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '^' => {
          v.push(Token{token_type: TokenType::Operator,
            value: TokenValue::Pow, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '.' => {
          v.push(Token{token_type: TokenType::Operator,
            value: TokenValue::Dot, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '<' => {
          if i+1<n && a[i+1]=='=' {
            v.push(Token{token_type: TokenType::Operator,
              value: TokenValue::Le, line: line, col: col, s: None});
            i+=2; col+=2;
          }else{
            v.push(Token{token_type: TokenType::Operator,
              value: TokenValue::Lt, line: line, col: col, s: None});
            i+=1; col+=1;
          }
        },
        '>' => {
          if i+1<n && a[i+1]=='=' {
            v.push(Token{token_type: TokenType::Operator,
              value: TokenValue::Ge, line: line, col: col, s: None});
            i+=2; col+=2;
          }else{
            v.push(Token{token_type: TokenType::Operator,
              value: TokenValue::Gt, line: line, col: col, s: None});
            i+=1; col+=1;
          }
        },
        '|' => {
          v.push(Token{token_type: TokenType::Operator,
            value: TokenValue::Vline, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '&' => {
          v.push(Token{token_type: TokenType::Operator,
            value: TokenValue::Amp, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '$' => {
          v.push(Token{token_type: TokenType::Operator,
            value: TokenValue::Svert, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '~' => {
          v.push(Token{token_type: TokenType::Operator,
            value: TokenValue::Tilde, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '"' => {
          hcol=col;
          i+=1; col+=1;
          let j=i;
          while i<n && a[i]!='"' {i+=1; col+=1;}
          let s: &String = &a[j..i].iter().cloned().collect();
          v.push(Token{token_type: TokenType::String,
            value: TokenValue::None, line: line, col: hcol, s: Some(s.clone())
          });
          i+=1; col+=1;
        },
        '!' => {
          if i+1<n && a[i+1]=='=' {
            v.push(Token{token_type: TokenType::Operator,
              value: TokenValue::Ne, line: line, col: col, s: None});
            i+=2; col+=2;
          }else{
            return Err(SyntaxError{line: line, col: col,
              s: format!("unexpected character '{}'.", c)});          
          }
        },
        _ => {
          return Err(SyntaxError{line: line, col: col,
            s: format!("unexpected character '{}'.", c)});
        }
      }
    }
  }
  v.push(Token{token_type: TokenType::Terminal,
    value: TokenValue::None, line: line, col: col, s: None});
  return Ok(v);
}

fn token_value_to_string(value: &TokenValue) -> &'static str {
  return match *value {
    TokenValue::Plus => "+",  TokenValue::Minus => "-",
    TokenValue::Ast  => "*",  TokenValue::Div => "/",
    TokenValue::Mod  => "%",  TokenValue::Pow => "^",
    TokenValue::Vline=> "|",  TokenValue::Amp => "&",
    TokenValue::Idiv => "//", TokenValue::Svert=> "$",
    TokenValue::In   => "in", TokenValue::Is => "is",
    TokenValue::And  => "and",TokenValue::Or => "or",
    TokenValue::Not  => "not",TokenValue::Tilde => "~",
    TokenValue::PLeft => "(", TokenValue::PRight => ")",
    TokenValue::BLeft => "[", TokenValue::BRight => "]",
    TokenValue::CLeft => "{", TokenValue::CRight => "}",
    TokenValue::Lt    => "<", TokenValue::Gt => ">",
    TokenValue::Le   => "<=", TokenValue::Ge => ">=",
    TokenValue::Dot   => ".", TokenValue::Comma => ",",
    TokenValue::Colon => ":", TokenValue::Semicolon => ";",
    TokenValue::Eq   => "==", TokenValue::Ne => "!=",
    TokenValue::List => "[]", TokenValue::Application => "app",
    TokenValue::Map  => "{}", TokenValue::Index => "index",
    TokenValue::Assignment => "=",
    TokenValue::Newline => "\\n",
    TokenValue::Assert => "assert",
    TokenValue::Begin => "begin",
    TokenValue::Break => "break",
    TokenValue::Catch => "catch",
    TokenValue::Continue => "continue",
    TokenValue::Elif => "elif",
    TokenValue::Else => "else",
    TokenValue::End => "end",
    TokenValue::False => "false",
    TokenValue::For => "for",
    TokenValue::Global => "global",
    TokenValue::Goto => "goto",
    TokenValue::If => "if",
    TokenValue::Null => "null",
    TokenValue::Raise => "raise",
    TokenValue::Return => "return",
    TokenValue::Sub => "sub",
    TokenValue::Table => "table",
    TokenValue::Then => "then",
    TokenValue::True => "true",
    TokenValue::Try => "try",
    TokenValue::Use => "use",
    TokenValue::While => "while",
    TokenValue::Yield => "yield",
    _ => "unknown token value"
  };
}

fn print_token(x: &Token){
  match x.token_type {
    TokenType::String | TokenType::Int | TokenType::Identifier => {
      print!("[{}]",match x.s {Some(ref s) => s, None => compiler_error()});
    },
    TokenType::Operator | TokenType::Separator |
    TokenType::Bracket  | TokenType::Keyword | TokenType::Bool => {
      print!("[{}]",token_value_to_string(&x.value));
    },
    TokenType::Terminal => {
      print!("[Terminal]");
    }
  }
}

pub fn print_vtoken(v: &Vec<Token>){
  for x in v {print_token(x);}
  println!();
}

fn print_ast(t: &ASTNode, indent: usize){
  print!("{:1$}","",indent);
  match t.token_type {
    TokenType::Identifier | TokenType::Int => {
      println!("{}",match t.s {Some(ref s) => s, None => compiler_error()});
    },
    TokenType::String => {
      println!("\"{}\"",match t.s {Some(ref s) => s, None => compiler_error()});    
    },
    TokenType::Operator | TokenType::Separator |
    TokenType::Keyword  | TokenType::Bool => {
      println!("{}",token_value_to_string(&t.value));
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

fn scan_line(line_start: usize) -> Result<Vec<Token>,SyntaxError>{
  let input = match system::getline("| ") {
    Ok(x) => x,
    Err(x) => panic!()
  };
  return scan(&input,line_start);
}

struct ASTNode{
  line: usize, col: usize,
  token_type: TokenType,
  value: TokenValue,
  s: Option<String>,
  a: Option<Box<[Rc<ASTNode>]>>
}

pub struct Compilation<'a>{
  mode_cmd: bool,
  index: usize,
  parens: bool,
  history: &'a system::History
}

struct TokenIterator{
  pub a: Rc<Box<[Token]>>,
  pub index: usize
}

impl TokenIterator{
  fn next_token(&mut self, c: &mut Compilation) -> Result<Rc<Box<[Token]>>,SyntaxError>{
    if c.mode_cmd {
      let token_type = self.a[self.index].token_type;
      if token_type==TokenType::Terminal {
        let line = self.a[self.index].line;
        let v = try!(scan_line(line+1));
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
  value: TokenValue, x: Rc<ASTNode>) -> Rc<ASTNode>
{
  return Rc::new(ASTNode{line: line, col: col, token_type: TokenType::Operator,
    value: value, s: None, a: Some(Box::new([x]))}); 
}

fn binary_operator(line: usize, col: usize, value: TokenValue,
  x: Rc<ASTNode>, y: Rc<ASTNode>) -> Rc<ASTNode>
{
  return Rc::new(ASTNode{line: line, col: col, token_type: TokenType::Operator,
    value: value, s: None, a: Some(Box::new([x,y]))}); 
}

impl<'a> Compilation<'a>{

fn list_literal(&mut self, i: &mut TokenIterator) -> Result<Box<[Rc<ASTNode>]>,SyntaxError> {
  let mut v: Vec<Rc<ASTNode>> = Vec::new();
  let p = try!(i.next_token(self));
  let t = &p[i.index];
  if t.value==TokenValue::BRight {
    i.index+=1;
    return Ok(v.into_boxed_slice());
  }
  loop{
    let x = try!(self.expression(i));
    v.push(x);
    let p = try!(i.next_token(self));
    let t = &p[i.index];
    if t.value==TokenValue::Comma {
      i.index+=1;
      let p = try!(i.next_token(self));
      let t = &p[i.index];
      if t.value==TokenValue::BRight {
        i.index+=1;
        break;
      }
    }else if t.value==TokenValue::BRight {
      i.index+=1;
      break;
    }else{
      return Err(SyntaxError{line: t.line, col: t.col, s: String::from("expected ',' or ']'.")});      
    }
  }
  return Ok(v.into_boxed_slice());
}

fn application(&mut self, i: &mut TokenIterator, f: Rc<ASTNode>, terminal: TokenValue)
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
    if t.value == TokenValue::Comma {
      i.index+=1;
    }else if t.value == terminal {
      i.index+=1;
      break;
    }else{
      return Err(SyntaxError{line: t.line, col: t.col, s: String::from("unexpected token.")});
    }
  }
  let value = if terminal==TokenValue::PRight
    {TokenValue::Application}
  else
    {TokenValue::Index};
  return Ok(Rc::new(ASTNode{line: line, col: col, token_type: TokenType::Operator,
    value: value, s: None, a: Some(v.into_boxed_slice())}));
}

fn atom(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError> {
  let p = try!(i.next_token(self));
  let t = &p[i.index];
  let y;
  if t.token_type==TokenType::Identifier || t.token_type==TokenType::Int ||
     t.token_type==TokenType::String
  {
    i.index+=1;
    y = Rc::new(ASTNode{line: t.line, col: t.col, token_type: t.token_type,
      value: TokenValue::Null, s: t.s.clone(), a: None});
  }else if t.value==TokenValue::PLeft {
    i.index+=1;
    self.parens = true;
    y = try!(self.expression(i));
    let p = try!(i.next_token(self));
    let t = &p[i.index];
    self.parens = false;
    if t.value != TokenValue::PRight {
      return Err(SyntaxError{line: t.line, col: t.col, s: String::from("expected ')'.")});
    }
    i.index+=1;
  }else if t.value==TokenValue::BLeft {
    i.index+=1;
    let x = try!(self.list_literal(i));
    y = Rc::new(ASTNode{line: t.line, col: t.col,
      token_type: TokenType::Operator, value: TokenValue::List,
      s: None, a: Some(x)
    });
  }else{
    return Err(SyntaxError{line: t.line, col: t.col, s: String::from("unexpected token.")});
  }
  let p2 = try!(i.next_any_token(self));
  let t2 = &p2[i.index];
  if t2.value == TokenValue::PLeft {
    i.index+=1;
    return self.application(i,y,TokenValue::PRight);
  }else if t2.value == TokenValue::BLeft {
    i.index+=1;
    return self.application(i,y,TokenValue::BRight);
  }else{
    return Ok(y);
  }
}

fn power(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
  let x = try!(self.atom(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value==TokenValue::Pow {
    i.index+=1;
    let y = try!(self.power(i));
    return Ok(binary_operator(t.line,t.col,TokenValue::Pow,x,y));
  }else{
    return Ok(x);
  }
}

fn signed_expression(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
  let p = try!(i.next_token(self));
  let t = &p[i.index];
  if t.value==TokenValue::Minus {
    i.index+=1;
    let x = try!(self.power(i));
    return Ok(unary_operator(t.line,t.col,TokenValue::Minus,x));
  }else{
    return self.power(i);
  }
}

fn factor(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
  let mut y = try!(self.signed_expression(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  let value=t.value;
  if value==TokenValue::Ast || value==TokenValue::Div || value==TokenValue::Idiv {
    i.index+=1;
    let x = try!(self.signed_expression(i));
    y = binary_operator(t.line,t.col,value,y,x);
    loop{
      let p = try!(i.next_any_token(self));
      let t = &p[i.index];
      let value = t.value;
      if value!=TokenValue::Ast && value!=TokenValue::Div && value!=TokenValue::Idiv {
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
  if value==TokenValue::Plus || value==TokenValue::Minus {
    i.index+=1;
    let x = try!(self.factor(i));
    y = binary_operator(t.line,t.col,value,y,x);
    loop{
      let p = try!(i.next_any_token(self));
      let t = &p[i.index];
      let value=t.value;
      if value!=TokenValue::Plus && value!=TokenValue::Minus {
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

fn comparison(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
  let x = try!(self.addition(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  let value=t.value;
  if value==TokenValue::Lt || value==TokenValue::Gt ||
     value==TokenValue::Le || value==TokenValue::Ge
  {
    i.index+=1;
    let y = try!(self.addition(i));
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
  if value==TokenValue::Eq || value==TokenValue::Ne ||
     value==TokenValue::Is || value==TokenValue::In
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
  if t.value==TokenValue::Not {
    i.index+=1;
    let x = try!(self.eq_expression(i));
    return Ok(unary_operator(t.line,t.col,TokenValue::Not,x));
  }else{
    return self.eq_expression(i);
  }
}

fn conjunction(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
  let x = try!(self.negation(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value==TokenValue::And {
    i.index+=1;
    let y = try!(self.negation(i));
    return Ok(binary_operator(t.line,t.col,TokenValue::And,x,y));
  }else{
    return Ok(x);
  }
}

fn disjunction(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
  let x = try!(self.conjunction(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value==TokenValue::Or {
    i.index+=1;
    let y = try!(self.conjunction(i));
    return Ok(binary_operator(t.line,t.col,TokenValue::Or,x,y));
  }else{
    return Ok(x);
  }
}

fn if_expression(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
  let x = try!(self.disjunction(i));
  let p = try!(i.next_any_token(self));
  let t = &p[i.index];
  if t.value==TokenValue::If {
    i.index+=1;
    let condition = try!(self.expression(i));
    let p2 = try!(i.next_any_token(self));
    let t2 = &p[i.index];
    if t2.value==TokenValue::Else {
      i.index+=1;
      let y = try!(self.expression(i));
      return Ok(Rc::new(ASTNode{
        line: t.line, col: t.col, token_type: TokenType::Operator,
        value: TokenValue::If, a: Some(Box::new([condition,x,y])), s: None
      }));
    }else{
      return Ok(binary_operator(t.line,t.col,TokenValue::If,condition,x));
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
  if t.value==TokenValue::Assignment {
    i.index+=1;
    let y = try!(self.expression(i));
    return Ok(binary_operator(t.line,t.col,TokenValue::Assignment,x,y));
  }else{
    return Ok(x);
  }
}

fn ast(&mut self, i: &mut TokenIterator) -> Result<Rc<ASTNode>,SyntaxError>{
  return self.assignment(i);
}

}//impl

pub fn compile(v: Vec<Token>, mode_cmd: bool, history: &mut system::History) -> Result<(),SyntaxError>{
  let mut compilation = Compilation{
    mode_cmd: mode_cmd, index: 0, parens: false,
    history: history
  };
  let mut i = TokenIterator{index: 0, a: Rc::new(v.into_boxed_slice())};
  let y = try!(compilation.ast(&mut i));
  print_ast(&y,2);
  return Ok(());
}
