
#![allow(dead_code)]

use std::rc::Rc;
use std::cell::Cell;
use std::fmt::Write;

pub struct Error {
    pub line: usize,
    pub col: usize,
    pub text: String
}

fn scan_error(line: usize, col: usize, text: String) -> Error {
    Error{text,line,col}
}

fn syntax_error(line: usize, col: usize, text: String) -> Error {
    let text = format!("Syntax error: {}",text);
    Error{text,line,col}
}

#[derive(Clone,Copy,PartialEq,Eq,Debug)]
enum Symbol {
    None, Terminal, Identifier, Int, String,
    Comma, Dot, Colon, Semicolon, Neg,
    Plus, Minus, Ast, Div, Pow, Mod, Idiv, Tilde, Amp, Vert, Svert,
    PLeft, PRight, BLeft, BRight, CLeft, CRight, Assignment, To,
    List, Application, Block,
    Assert, And, Begin, Break, Catch, Continue, Do, Elif, Else,
    End, False, For, Fn, Function, Global, Goto, Label, Let,
    If, In, Is, Not, Null, Of, Or, Public, Raise, Return,
    Table, Then, True, Try, Use, While, Yield
}

#[derive(Debug)]
enum Item {
    None, Int(i32), String(String)
}

pub struct Token {
    value: Symbol,
    item: Item,
    line: usize,
    col: usize
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.item {
            Item::None => write!(f, "{:?}", self.value),
            Item::Int(x) => write!(f, "{:?}({})", self.value, x),
            Item::String(s) =>  write!(f, "{:?}({})", self.value, s)
        }
    }
}

impl Token {
    fn symbol(line: usize, col: usize, value: Symbol) -> Self {
        Self{line, col, value, item: Item::None}
    }
}

struct KeywordsElement {
    s: &'static str,
    v: &'static Symbol
}

static KEYWORDS: &'static [KeywordsElement] = &[
      KeywordsElement{s: "assert",  v: &Symbol::Assert},
      KeywordsElement{s: "and",     v: &Symbol::And},
      KeywordsElement{s: "begin",   v: &Symbol::Begin},
      KeywordsElement{s: "break",   v: &Symbol::Break},
      KeywordsElement{s: "catch",   v: &Symbol::Catch},
      KeywordsElement{s: "continue",v: &Symbol::Continue},
      KeywordsElement{s: "do",      v: &Symbol::Do},
      KeywordsElement{s: "elif",    v: &Symbol::Elif},
      KeywordsElement{s: "else",    v: &Symbol::Else},
      KeywordsElement{s: "end",     v: &Symbol::End},
      KeywordsElement{s: "false",   v: &Symbol::False},
      KeywordsElement{s: "for",     v: &Symbol::For},
      KeywordsElement{s: "fn",      v: &Symbol::Fn},
      KeywordsElement{s: "function",v: &Symbol::Function},
      KeywordsElement{s: "global",  v: &Symbol::Global},
      KeywordsElement{s: "goto",    v: &Symbol::Goto},
      KeywordsElement{s: "label",   v: &Symbol::Label},
      KeywordsElement{s: "let",     v: &Symbol::Let},
      KeywordsElement{s: "if",      v: &Symbol::If},
      KeywordsElement{s: "in",      v: &Symbol::In},
      KeywordsElement{s: "is",      v: &Symbol::Is},
      KeywordsElement{s: "not",     v: &Symbol::Not},
      KeywordsElement{s: "null",    v: &Symbol::Null},
      KeywordsElement{s: "of",      v: &Symbol::Of},
      KeywordsElement{s: "or",      v: &Symbol::Or},
      KeywordsElement{s: "public",  v: &Symbol::Global},
      KeywordsElement{s: "raise",   v: &Symbol::Raise},
      KeywordsElement{s: "return",  v: &Symbol::Return},
      KeywordsElement{s: "table",   v: &Symbol::Table},
      KeywordsElement{s: "then",    v: &Symbol::Then},
      KeywordsElement{s: "true",    v: &Symbol::True},
      KeywordsElement{s: "try",     v: &Symbol::Try},
      KeywordsElement{s: "use",     v: &Symbol::Use},
      KeywordsElement{s: "while",   v: &Symbol::While},
      KeywordsElement{s: "yield",   v: &Symbol::Yield}
];

fn is_keyword(id: &String) -> Option<&'static KeywordsElement> {
    let n: usize = KEYWORDS.len();
    for i in 0..n {
        if KEYWORDS[i].s == id  {return Some(&KEYWORDS[i]);}
    }
    return None;
}

pub fn scan(s: &str) -> Result<Vec<Token>,Error> {
    let a: Vec<char> = s.chars().collect();
    let mut v: Vec<Token> = Vec::new();

    let mut i = 0;
    let n = a.len();
    let mut line = 0;
    let mut col = 0;
    
    while i<n {
        let c = a[i];
        if c.is_digit(10) {
            let j = i;
            while i<n && a[i].is_digit(10) {
                i+=1;
            }
            let s: String = a[j..i].iter().collect();
            let value = s.parse::<i32>().unwrap();
            v.push(Token{line, col,
                value: Symbol::Int, item: Item::Int(value)
            });
        }else if c.is_alphabetic() && c.is_ascii() || a[i]=='_' {
            let j = i;
            while i<n && (a[i].is_alphabetic() && a[i].is_ascii()
                || a[i].is_digit(10) || a[i]=='_'
            ) {
                i+=1; col+=1;
            }
            let id: String = a[j..i].iter().collect();
            if let Some(t) = is_keyword(&id) {
                v.push(Token::symbol(line,col,*t.v));
            }else{
                v.push(Token{line, col,
                    value: Symbol::Identifier, item: Item::String(id)
                });
            }
        }else{
            match c {
                ' ' => {
                    i+=1; col+=1;
                },
                '\n' => {
                    i+=1; line+=1;
                    col = 0;
                },
                ',' => {
                    v.push(Token::symbol(line,col,Symbol::Comma));
                    i+=1; col+=1;
                },
                '.' => {
                    v.push(Token::symbol(line,col,Symbol::Dot));
                    i+=1; col+=1;
                },
                ':' => {
                    v.push(Token::symbol(line,col,Symbol::Colon));
                    i+=1; col+=1;
                },
                ';' => {
                    v.push(Token::symbol(line,col,Symbol::Semicolon));
                    i+=1; col+=1;
                },
                '+' => {
                    v.push(Token::symbol(line,col,Symbol::Plus));
                    i+=1; col+=1;
                },
                '-' => {
                    v.push(Token::symbol(line,col,Symbol::Minus));
                    i+=1; col+=1;
                },
                '*' => {
                    v.push(Token::symbol(line,col,Symbol::Ast));
                    i+=1; col+=1;
                },
                '/' => {
                    v.push(Token::symbol(line,col,Symbol::Div));
                    i+=1; col+=1;
                },
                '%' => {
                    v.push(Token::symbol(line,col,Symbol::Mod));
                    i+=1; col+=1;
                },
                '^' => {
                    v.push(Token::symbol(line,col,Symbol::Pow));
                    i+=1; col+=1;
                },
                '~' => {
                    v.push(Token::symbol(line,col,Symbol::Tilde));
                    i+=1; col+=1;
                },
                '&' => {
                    v.push(Token::symbol(line,col,Symbol::Amp));
                    i+=1; col+=1;
                },
                '|' => {
                    v.push(Token::symbol(line,col,Symbol::Vert));
                    i+=1; col+=1;
                },
                '$' => {
                    v.push(Token::symbol(line,col,Symbol::Svert));
                    i+=1; col+=1;
                },
                '(' => {
                    v.push(Token::symbol(line,col,Symbol::PLeft));
                    i+=1; col+=1;
                },
                ')' => {
                    v.push(Token::symbol(line,col,Symbol::PRight));
                    i+=1; col+=1;
                },
                '[' => {
                    v.push(Token::symbol(line,col,Symbol::BLeft));
                    i+=1; col+=1;
                },
                ']' => {
                    v.push(Token::symbol(line,col,Symbol::BRight));
                    i+=1; col+=1;
                },
                '{' => {
                    v.push(Token::symbol(line,col,Symbol::CLeft));
                    i+=1; col+=1;
                },
                '}' => {
                    v.push(Token::symbol(line,col,Symbol::CRight));
                    i+=1; col+=1;
                },
                '=' => {
                    if i+1<n && a[i+1]=='>' {
                        v.push(Token::symbol(line,col,Symbol::To));
                        i+=2; col+=2;
                    }else{
                        v.push(Token::symbol(line,col,Symbol::Assignment));
                        i+=1; col+=1;
                    }
                },
                '#' => {
                    while i<n && a[i]!='\n' {i+=1;}
                    i+=1; col=0;
                },
                '"' => {
                    i+=1; col+=1;
                    let j = i;
                    while i<n && a[i]!='"' {
                        if a[i]=='\n' {line+=1; col=0;}
                        else {col+=1;}
                        i+=1;
                    }
                    let literal: String = a[j..i].iter().collect();
                    v.push(Token{line, col,
                        value: Symbol::String, item: Item::String(literal)
                    });
                    i+=1;
                },
                _ => {
                    return Err(scan_error(line,col,format!("Unexpected character: '{}'.",c)));
                }
            }
        }
    }
    v.push(Token{line, col, value: Symbol::Terminal, item: Item::None});
    return Ok(v);
}

struct TokenIterator<'a> {
    a: &'a [Token],
    index: Cell<usize>
}

impl<'a> TokenIterator<'a> {
    fn new(a: &'a [Token]) -> TokenIterator<'a> {
        TokenIterator{a: a, index: Cell::new(0)}
    }
    fn get(&self) -> &Token {
        return &self.a[self.index.get()];
    }
    fn advance(&self) {
        self.index.set(self.index.get()+1);
    }
}

#[derive(Debug)]
enum Info {
    None, Int(i32), String(String)
}

#[derive(Debug)]
struct AST {
    line: usize,
    col: usize,
    value: Symbol,
    info: Info,
    a: Option<Box<[Rc<AST>]>>
}

impl AST {
    fn node(line: usize, col: usize, value: Symbol,
        info: Info, a: Option<Box<[Rc<AST>]>>
    ) -> Rc<AST>
    {
        Rc::new(AST{line,col,value,info,a})
    }

    fn operator(line: usize, col: usize, value: Symbol, a: Box<[Rc<AST>]>
    ) -> Rc<AST>
    {
        Rc::new(AST{line,col,value,info: Info::None, a: Some(a)})
    }
}

const INDENT_SHIFT: usize = 4;

fn ast_to_string(buffer: &mut String, t: &AST, indent: usize) {
    write!(buffer,"{: <1$}","",indent).ok();
    if t.value == Symbol::Identifier {
        let id = match &t.info {Info::String(s) => s, _ => unreachable!()};
        write!(buffer,"{:?}({})\n",t.value,id).ok();
    }else if t.value == Symbol::String {
        let s = match &t.info {Info::String(s) => s, _ => unreachable!()};
        write!(buffer,"\"{}\"\n",s).ok();
    }else if t.value == Symbol::Int {
        let x = match &t.info {Info::Int(x) => x, _ => unreachable!()};
        write!(buffer,"Int({})\n",x).ok();
    }else{
        write!(buffer,"{:?}\n",t.value).ok();
    }
    if let Some(a) = &t.a {
        for x in a.iter() {
            ast_to_string(buffer,x,indent+INDENT_SHIFT);
        }
    }
}

fn print_ast(t: &AST){
    let mut buffer = String::new();
    ast_to_string(&mut buffer,t,INDENT_SHIFT);
    println!("{}",buffer);
}

fn expect_string(x: &Item) -> String {
    match x {Item::String(s) => s.clone(), _ => unreachable!()}
}

fn expect_int(x: &Item) -> i32 {
    match x {Item::Int(x) => *x, _ => unreachable!()}
}

fn lambda_expression(t0: &Token, i: &TokenIterator) -> Result<Rc<AST>,Error> {
    let mut argv: Vec<Rc<AST>> = Vec::new();
    loop{
        let x = atom(i)?;
        argv.push(x);
        let t = i.get();
        if t.value == Symbol::Vert {
            i.advance();
            break;
        }else if t.value == Symbol::Comma {
            i.advance();
        }else{
            return Err(syntax_error(t.line,t.col,
                String::from("expected ',' or '|'.")
            ));
        }
    }
    let arg_list = AST::node(t0.line, t0.col, Symbol::List,
        Info::None, Some(argv.into_boxed_slice())
    );
    let x = expression(i)?;
    return Ok(AST::node(t0.line, t0.col, Symbol::Fn,
        Info::None, Some(Box::new([arg_list,x]))
    ));
}

fn atom(i: &TokenIterator) -> Result<Rc<AST>,Error> {
    let t = i.get();
    if t.value == Symbol::Identifier || t.value == Symbol::String {
        i.advance();
        return Ok(AST::node(t.line,t.col,t.value,
            Info::String(expect_string(&t.item)), None
        ));
    }else if t.value == Symbol::Int {
        i.advance();
        return Ok(AST::node(t.line,t.col,t.value,
            Info::Int(expect_int(&t.item)), None
        ));
    }else if t.value == Symbol::PLeft {
        i.advance();
        let x = expression(i)?;
        let t = i.get();
        if t.value == Symbol::PRight {
            i.advance();
        }else{
            return Err(syntax_error(t.line,t.col,
                String::from("expected symbol ')'")
            ));
        }
        return Ok(x);
    }else if t.value == Symbol::Vert {
        i.advance();
        return lambda_expression(t,i);
    }else{
        return Err(syntax_error(t.line,t.col,
            format!("unexpected symbol: '{:?}'",t.value)
        ));
    }
}

fn argument_list(t0: &Token, i: &TokenIterator, terminator: Symbol)
-> Result<Rc<AST>,Error>
{
    let mut argv: Vec<Rc<AST>> = Vec::new();
    loop{
        let x = expression(i)?;
        argv.push(x);
        let t = i.get();
        if t.value == terminator {
            break;
        }else if t.value == Symbol::Comma {
            i.advance();
        }else{
            return Err(syntax_error(t.line,t.col,
                format!("unexpected symbol: '{:?}'",t.value)
            ));
        }
    }
    i.advance();
    return Ok(AST::node(t0.line, t0.col, Symbol::List,
        Info::None, Some(argv.into_boxed_slice())
    ));
}

fn application(i: &TokenIterator) -> Result<Rc<AST>,Error> {
    let x = atom(i)?;
    let t = i.get();
    if t.value == Symbol::PLeft {
        i.advance();
        let argv = argument_list(&t,i,Symbol::PRight)?;
        return Ok(AST::node(t.line, t.col, Symbol::Application,
            Info::None, Some(Box::new([x, argv]))
        ));
    }
    return Ok(x);
}

fn power(i: &TokenIterator) -> Result<Rc<AST>,Error> {
    let x = application(i)?;
    let t = i.get();
    if t.value == Symbol::Pow {
        i.advance();
        let y = power(i)?;
        return Ok(AST::operator(t.line,t.col,t.value,Box::new([x,y])));
    }else{
        return Ok(x);
    }
}

fn negation(i: &TokenIterator) -> Result<Rc<AST>,Error> {
    let t = i.get();
    if t.value == Symbol::Minus {
        i.advance();
        let x = power(i)?;
        return Ok(AST::operator(t.line,t.col,Symbol::Neg,Box::new([x])));
    }else{
        return power(i);
    }
}

fn multiplication(i: &TokenIterator) -> Result<Rc<AST>,Error> {
    let mut x = negation(i)?;
    loop{
        let t = i.get();
        if t.value == Symbol::Ast  || t.value == Symbol::Div ||
           t.value == Symbol::Idiv || t.value == Symbol::Mod
        {
            i.advance();
            let y = negation(i)?;
            x = AST::operator(t.line,t.col,t.value,Box::new([x,y]));
        }else{
            return Ok(x);
        }
    }
}

fn addition(i: &TokenIterator) -> Result<Rc<AST>,Error> {
    let mut x = multiplication(i)?;
    loop{
        let t = i.get();
        if t.value == Symbol::Plus || t.value == Symbol::Minus {
            i.advance();
            let y = multiplication(i)?;
            x = AST::operator(t.line,t.col,t.value,Box::new([x,y]));
        }else{
            return Ok(x);
        }
    }
}

fn expression(i: &TokenIterator) -> Result<Rc<AST>,Error> {
    return addition(i);
}

fn expect_semicolon(i: &TokenIterator) -> Result<(),Error> {
    let t = i.get();
    if t.value == Symbol::Semicolon {
        i.advance();
        return Ok(());
    }else{
        return Err(syntax_error(t.line,t.col,
            String::from("expected semicolon")
        ));
    }
}

fn statements(i: &TokenIterator) -> Result<Rc<AST>,Error> {
    let mut a: Vec<Rc<AST>> = Vec::new();
    let t0 = i.get();
    loop{
        let t = i.get();
        let line = t.line;
        let col = t.col;
        if t.value == Symbol::Let {
            i.advance();
            let id = atom(i)?;

            let t = i.get();
            let texp = if t.value == Symbol::Colon {
                i.advance();
                type_expression(i)?
            }else{
                AST::node(t.line,t.col,Symbol::None,Info::None,None)
            };

            let t = i.get();
            if t.value == Symbol::Assignment {
                i.advance();
                let x = expression(i)?;
                let let_exp = AST::node(line,col,Symbol::Let,Info::None,
                    Some(Box::new([id,texp,x]))
                );
                a.push(let_exp);
                expect_semicolon(i)?;
            }else if t.value == Symbol::Semicolon {
                i.advance();
                let let_exp = AST::node(line,col,Symbol::Let,Info::None,
                    Some(Box::new([id,texp]))
                );
                a.push(let_exp);
            }else{
                return Err(syntax_error(t.line,t.col,
                    String::from("expected '='")
                ));
            }
        }else if t.value == Symbol::Terminal {
            break;
        }else{
            let x = expression(i)?;
            a.push(x);
            expect_semicolon(i)?;
        }
    }
    return Ok(AST::node(t0.line, t0.col, Symbol::Block,
        Info::None, Some(a.into_boxed_slice())
    ));
}

fn type_atom(i: &TokenIterator) -> Result<Rc<AST>,Error> {
    let t = i.get();
    if t.value == Symbol::Identifier {
        i.advance();
        return Ok(AST::node(t.line,t.col,t.value,
            Info::String(expect_string(&t.item)), None
        ));
    }else if t.value == Symbol::PLeft {
        i.advance();
        let x = type_expression(i)?;
        let t = i.get();
        if t.value == Symbol::PRight {
            i.advance();
            return Ok(x);
        }else if t.value == Symbol::Comma {
            i.advance();
            let mut v: Vec<Rc<AST>> = Vec::with_capacity(2);
            v.push(x);
            return type_argument_list(t,v,i,Symbol::PRight);
        }else{
            return Err(syntax_error(t.line,t.col,
                String::from("expected ',' or ')'.")
            ));        
        }
    }else{
        return Err(syntax_error(t.line,t.col,
            format!("unexpected symbol: '{:?}'.",t.value)
        ));
    }
}

fn type_argument_list(t0: &Token, mut argv: Vec<Rc<AST>>,
  i: &TokenIterator, terminator: Symbol
) -> Result<Rc<AST>,Error>
{
    loop{
        let x = type_expression(i)?;
        argv.push(x);
        let t = i.get();
        if t.value == terminator {
            break;
        }else if t.value == Symbol::Comma {
            i.advance();
        }else{
            return Err(syntax_error(t.line,t.col,
                format!("unexpected symbol: '{:?}'",t.value)
            ));
        }
    }
    i.advance();
    return Ok(AST::node(t0.line, t0.col, Symbol::List,
        Info::None, Some(argv.into_boxed_slice())
    ));
}

fn type_application(i: &TokenIterator) -> Result<Rc<AST>,Error> {
    let x = type_atom(i)?;
    let t = i.get();
    if t.value == Symbol::BLeft {
        i.advance();
        let argv = type_argument_list(t,Vec::new(),i,Symbol::BRight)?;
        return Ok(AST::node(t.line, t.col, Symbol::Application,
            Info::None, Some(Box::new([x,argv]))
        ));
    }else{
        return Ok(x);
    }
}

fn type_fn(i: &TokenIterator) -> Result<Rc<AST>,Error> {
    let x = type_application(i)?;
    let t = i.get();
    if t.value == Symbol::To {
        i.advance();
        let y = type_fn(i)?;
        return Ok(AST::node(t.line, t.col, Symbol::Fn,
            Info::None, Some(Box::new([x,y]))
        ));
    }else{
        return Ok(x);
    }
}

fn type_expression(i: &TokenIterator) -> Result<Rc<AST>,Error> {
    return type_fn(i);
}

pub fn parse(s: &str) -> Result<(),Error> {
    let v = scan(s)?;
    println!("{:?}\n",v);
    let i = TokenIterator::new(&v);
    let x = statements(&i)?;
    print_ast(&x);
    return Ok(());
}


