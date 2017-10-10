
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

// use std::io;
// use std::io::Write;
use std::ascii::AsciiExt;

#[derive(Copy,Clone)]
enum TokenType{
  Terminal, Operator, Separator, Bracket, Bool, Int,
  String, Identifier, Keyword
}

#[derive(Copy,Clone)]
enum TokenValue{
  None, Plus, Minus, Ast, Div, Idiv, Mod, Pow, Lt, Gt, In, Is,
  And, Or, Amp, Vline, Not, Tilde, Svert, Assignment,
  PLeft, PRight, BLeft, BRight, CLeft, CRight, Newline,
  Assert, Begin, Break, Catch, Continue,
  Elif, Else, End, For, Global, Goto, Label,
  If, While, Do, Raise, Return, Sub, Table, Then, Try,
  Use, Yield, True, False, Null, Dot, Comma, Colon, Semicolon
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

pub fn scan(s: &String) -> Result<Vec<Token>, SyntaxError>{
  let mut v: Vec<Token> = Vec::new();
  let mut line=1;
  let mut col=1;
  let a: Vec<char> = s.chars().collect();
  let mut i=0;
  let n = a.len();
  while i<n {
    let c = a[i];
    if c.is_digit(10) {
      let j=i;
      while i<n && a[i].is_digit(10) {
        i+=1; col+=1;
      }
      let number: &String = &a[j..i].iter().cloned().collect();
      v.push(Token{token_type: TokenType::Int,
        value: TokenValue::None, line: line, col: col, s: Some(number.clone())});
    }else if (c.is_alphabetic() && c.is_ascii()) || a[i]=='_' {
      let j=i;
      while i<n && (a[i].is_alphabetic() || a[i].is_digit(10) || a[i]=='_') {
        i+=1; col+=1;
      }
      let id: &String = &a[j..i].iter().cloned().collect();
      match is_keyword(id) {
        Some(x) => {
          v.push(Token{token_type: *x.t, value: *x.v,
            line: line, col: col, s: None});
        },
        None => {
          v.push(Token{token_type: TokenType::Identifier,
            value: TokenValue::None, line: line, col: col, s: Some(id.clone())});
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
          v.push(Token{token_type: TokenType::Operator,
            value: TokenValue::Assignment, line: line, col: col, s: None});
          i+=1; col+=1;
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
          v.push(Token{token_type: TokenType::Operator,
            value: TokenValue::Lt, line: line, col: col, s: None});
          i+=1; col+=1;
        },
        '>' => {
          v.push(Token{token_type: TokenType::Operator,
            value: TokenValue::Gt, line: line, col: col, s: None});
          i+=1; col+=1;
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
    TokenValue::Dot   => ".", TokenValue::Comma => ",",
    TokenValue::Colon => ":", TokenValue::Semicolon => ";",
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

pub fn print_vtoken(v: &Vec<Token>){
  for x in v {
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
  println!();
}

