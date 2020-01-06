
#![allow(dead_code)]

use std::rc::Rc;
use std::cell::Cell;
use std::fmt::Write;
use crate::typing::Type;

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
pub enum Symbol {
    None, Terminal, Item,
    Comma, Dot, Colon, Semicolon, Neg,
    Plus, Minus, Ast, Div, Pow, Mod, Idiv, Tilde, Amp, Vert, Svert,
    Lt, Le, Gt, Ge, Eq, Ne, Cond, Index, Range,
    PLeft, PRight, BLeft, BRight, CLeft, CRight, Assignment, To,
    List, Application, Block, Statement, Unit,
    As, Assert, And, Begin, Break, Catch, Continue, Do, Elif, Else,
    End, False, For, Fn, Function, Global, Goto, Label, Let,
    If, In, Is, Mut, Not, Null, Of, Or, Public, Raise, Return,
    Table, Then, True, Try, Use, While, Yield
}

#[derive(Debug)]
pub enum Item {
    None, Int(i32), Id(String), String(String)
}

#[derive(Copy,Clone,PartialEq)]
pub enum Value{
    None, Optional, Null
}

pub struct Token {
    pub value: Symbol,
    pub item: Item,
    pub line: usize,
    pub col: usize
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.item {
            Item::None => write!(f, "{:?}", self.value),
            Item::Int(x) => write!(f, "{:?}({})", self.value, x),
            Item::Id(id) => write!(f, "{:?}({})", self.value, id),
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
      KeywordsElement{s: "as",      v: &Symbol::As},
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
      KeywordsElement{s: "mut",     v: &Symbol::Mut},
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

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Symbol::None  => write!(f,"None"),
            Symbol::Terminal => write!(f,"Terminal"),
            Symbol::Item  => write!(f,"Item"),
            Symbol::Neg   => write!(f,"-"),
            Symbol::Plus  => write!(f,"+"),
            Symbol::Minus => write!(f,"-"),
            Symbol::Ast   => write!(f,"*"),
            Symbol::Div   => write!(f,"/"),
            Symbol::Pow   => write!(f,"^"),
            Symbol::Mod   => write!(f,"%"),
            Symbol::Idiv  => write!(f,"//"),
            Symbol::Tilde => write!(f,"~"),
            Symbol::Amp   => write!(f,"&"),
            Symbol::Vert  => write!(f,"|"),
            Symbol::Svert => write!(f,"$"),
            Symbol::Lt    => write!(f,"<"),
            Symbol::Le    => write!(f,"<="),
            Symbol::Gt    => write!(f,">"),
            Symbol::Ge    => write!(f,">="),
            Symbol::Eq    => write!(f,"=="),
            Symbol::Ne    => write!(f,"!="),
            Symbol::Cond  => write!(f,"cond"),
            Symbol::Index => write!(f,"index"),
            Symbol::Range => write!(f,".."),
            Symbol::Comma => write!(f,","),
            Symbol::Dot   => write!(f,"."),
            Symbol::Colon => write!(f,":"),
            Symbol::Semicolon => write!(f,";"),
            Symbol::PLeft => write!(f,"("),
            Symbol::PRight=> write!(f,")"),
            Symbol::BLeft => write!(f,"["),
            Symbol::BRight=> write!(f,"]"),
            Symbol::CLeft => write!(f,"{{"),
            Symbol::CRight=> write!(f,"}}"),
            Symbol::Assignment => write!(f,"="),
            Symbol::To    => write!(f,"=>"),
            Symbol::List  => write!(f,"List"),
            Symbol::Application => write!(f,"Application"),
            Symbol::Block => write!(f,"Block"),
            Symbol::Statement => write!(f,"Statement"),
            Symbol::Unit  => write!(f,"Unit"),
            Symbol::As    => write!(f,"as"),
            Symbol::Assert=> write!(f,"assert"),
            Symbol::And   => write!(f,"and"),
            Symbol::Begin => write!(f,"begin"),
            Symbol::Break => write!(f,"break"),
            Symbol::Catch => write!(f,"catch"),
            Symbol::Continue => write!(f,"continue"),
            Symbol::Do    => write!(f,"do"),
            Symbol::Elif  => write!(f,"elif"),
            Symbol::Else  => write!(f,"else"),
            Symbol::End   => write!(f,"end"),
            Symbol::False => write!(f,"false"),
            Symbol::For   => write!(f,"for"),
            Symbol::Fn    => write!(f,"fn"),
            Symbol::Function => write!(f,"function"),
            Symbol::Global=> write!(f,"global"),
            Symbol::Goto  => write!(f,"goto"),
            Symbol::Label => write!(f,"label"),
            Symbol::Let   => write!(f,"let"),
            Symbol::If    => write!(f,"if"),
            Symbol::In    => write!(f,"in"),
            Symbol::Is    => write!(f,"is"),
            Symbol::Mut   => write!(f,"mut"),
            Symbol::Not   => write!(f,"not"),
            Symbol::Null  => write!(f,"null"),
            Symbol::Of    => write!(f,"of"),
            Symbol::Or    => write!(f,"or"),
            Symbol::Public=> write!(f,"public"),
            Symbol::Raise => write!(f,"raise"),
            Symbol::Return=> write!(f,"return"),
            Symbol::Table => write!(f,"table"),
            Symbol::Then  => write!(f,"then"),
            Symbol::True  => write!(f,"true"),
            Symbol::Try   => write!(f,"try"),
            Symbol::Use   => write!(f,"use"),
            Symbol::While => write!(f,"while"),
            Symbol::Yield => write!(f,"yield")
        }
    }
}

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
            let hcol = col;
            let j = i;
            while i<n && a[i].is_digit(10) {
                i+=1;
            }
            let s: String = a[j..i].iter().collect();
            let value = s.parse::<i32>().unwrap();
            v.push(Token{line, col: hcol,
                value: Symbol::Item, item: Item::Int(value)
            });
        }else if c.is_alphabetic() && c.is_ascii() || a[i]=='_' {
            let hcol = col;
            let j = i;
            while i<n && (a[i].is_alphabetic() && a[i].is_ascii()
                || a[i].is_digit(10) || a[i]=='_'
            ) {
                i+=1; col+=1;
            }
            let id: String = a[j..i].iter().collect();
            if let Some(t) = is_keyword(&id) {
                v.push(Token::symbol(line,hcol,*t.v));
            }else{
                v.push(Token{line, col: hcol,
                    value: Symbol::Item, item: Item::Id(id)
                });
            }
        }else{
            match c {
                ' ' => {
                    i+=1; col+=1;
                },
                '\r' => {i+=1;},
                '\n' => {
                    i+=1; line+=1;
                    col = 0;
                },
                ',' => {
                    v.push(Token::symbol(line,col,Symbol::Comma));
                    i+=1; col+=1;
                },
                '.' => {
                    if i+1<n && a[i+1]=='.' {
                        v.push(Token::symbol(line,col,Symbol::Range));
                        i+=2; col+=2;
                    }else{
                        v.push(Token::symbol(line,col,Symbol::Dot));
                        i+=1; col+=1;
                    }
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
                    if i+1<n && a[i+1]=='*' {
                        while i+1<n && (a[i]!='*' || a[i+1]!='/') {
                            if a[i]=='\n' {col=0; line+=1;}
                            else {col+=1;}
                            i+=1;
                        }
                        i+=2; col+=2;
                    }else{
                        v.push(Token::symbol(line,col,Symbol::Div));
                        i+=1; col+=1;
                    }
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
                '<' => {
                    if i+1<n && a[i+1]=='=' {
                        v.push(Token::symbol(line,col,Symbol::Le));
                        i+=2; col+=2;
                    }else{
                        v.push(Token::symbol(line,col,Symbol::Lt));
                        i+=1; col+=1;
                    }
                },
                '>' => {
                    if i+1<n && a[i+1]=='=' {
                        v.push(Token::symbol(line,col,Symbol::Ge));
                        i+=2; col+=2;
                    }else{
                        v.push(Token::symbol(line,col,Symbol::Gt));
                        i+=1; col+=1;
                    }
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
                    }else if i+1<n && a[i+1]=='=' {
                        v.push(Token::symbol(line,col,Symbol::Eq));
                        i+=2; col+=2;
                    }else{
                        v.push(Token::symbol(line,col,Symbol::Assignment));
                        i+=1; col+=1;
                    }
                },
                '#' => {
                    while i<n && a[i]!='\n' {i+=1;}
                    i+=1; col=0; line+=1;
                },
                '"' => {
                    i+=1; col+=1;
                    let j = i;
                    while i<n && a[i]!='"' {
                        if a[i]=='\n' {col=0; line+=1;}
                        else {col+=1;}
                        i+=1;
                    }
                    let literal: String = a[j..i].iter().collect();
                    v.push(Token{line, col,
                        value: Symbol::Item, item: Item::String(literal)
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

const SYMBOL_TABLE_DANGLING: usize = std::usize::MAX;

pub struct FnHeader {
    pub argv: Vec<Argument>,
    pub id: Option<String>,
    pub ret_type: Rc<AST>,
    pub type_variables: Option<Rc<AST>>,
    pub symbol_table_index: Cell<usize>
}

pub enum Info {
    None, Mut, Int(i32), Id(String),
    String(String), FnHeader(Box<FnHeader>),
}

pub struct TypeRef {
    pub index: Cell<usize>
}
impl TypeRef {
    fn none() -> Self {
        Self{index: Cell::new(0)}
    }
}

pub struct AST {
    pub line: usize,
    pub col: usize,
    pub value: Symbol,
    pub info: Info,
    pub typ: TypeRef,
    pub a: Option<Box<[Rc<AST>]>>
}

impl AST {
    pub fn node(line: usize, col: usize, value: Symbol,
        info: Info, a: Option<Box<[Rc<AST>]>>
    ) -> Rc<AST>
    {
        Rc::new(AST{line,col,value,info,a, typ: TypeRef::none()})
    }

    pub fn operator(line: usize, col: usize, value: Symbol, a: Box<[Rc<AST>]>
    ) -> Rc<AST>
    {
        Rc::new(AST{line,col,value,info: Info::None, a: Some(a), typ: TypeRef::none()})
    }
    
    pub fn symbol(line: usize, col: usize, value: Symbol) -> Rc<AST> {
        Rc::new(AST{line,col,value,info: Info::None, a: None, typ: TypeRef::none()})
    }

    pub fn apply(line: usize, col: usize, a: Box<[Rc<AST>]>) -> Rc<AST> {
        Rc::new(AST{line,col,value: Symbol::Application,
            info: Info::None, a: Some(a), typ: TypeRef::none()})
    }
    
    pub fn identifier(id: &str, line: usize, col: usize) -> Rc<AST> {
        Rc::new(AST{line,col,value: Symbol::Item,
            info: Info::Id(id.into()), a: None, typ: TypeRef::none()})
    }

    pub fn argv(&self) -> &[Rc<AST>] {
        if let Some(a) = &self.a {
            return a;
        }else{
            unreachable!();
        }
    }

    pub fn string(&self,types: &Vec<Type>) -> String {
        let mut buffer = String::new();
        ast_to_string(&mut buffer,self,INDENT_START,Some(types));
        return buffer;
    }
}

pub struct Argument {
    pub id: String,
    pub ty: Rc<AST>
}

const INDENT_START: usize = 4;
const INDENT_SHIFT: usize = 4;

fn ast_to_string(buffer: &mut String, t: &AST, indent: usize,
    types: Option<&Vec<Type>>
) {
    write!(buffer,"{: <1$}","",indent).ok();
    if t.value == Symbol::Item {
        match t.info {
            Info::Id(ref id) => {
                write!(buffer,"Id({})",id).ok();
            },
            Info::String(ref s) => {
                write!(buffer,"\"{}\"",s).ok();
            },
            Info::Int(ref x) => {
                write!(buffer,"{}",x).ok();
            },
            _ => unreachable!()
        }
    }else{
        write!(buffer,"{}",t.value).ok();
    }
    if let Some(types) = types {
        let index = t.typ.index.get();
        if index != 0 {
            write!(buffer,": {}",&types[index]).ok();
        }
    }
    write!(buffer,"\n").ok();
    if let Some(a) = &t.a {
        for x in a.iter() {
            ast_to_string(buffer,x,indent+INDENT_SHIFT,types);
        }
    }
}

impl std::fmt::Display for AST {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut buffer = String::new();
        ast_to_string(&mut buffer,self,INDENT_START,None);
        return write!(f,"{}",buffer);
    }
}

fn expect_string(x: &Item) -> String {
    match x {Item::String(s) => s.clone(), _ => unreachable!()}
}

fn expect_int(x: &Item) -> i32 {
    match x {Item::Int(x) => *x, _ => unreachable!()}
}

fn identifier_from_string(id: String, line: usize, col: usize) -> Rc<AST> {
    return AST::node(line,col,Symbol::Item,Info::Id(id),None);
}

struct Parser{}

impl Parser {

fn lambda_formal_argument_list(&mut self, i: &TokenIterator)
-> Result<Vec<Argument>,Error>
{
    let mut argv: Vec<Argument> = Vec::new();
    let t = i.get();
    if t.value == Symbol::Vert {
        i.advance();
        return Ok(argv);
    }
    loop{
        let id = self.identifier_raw(i)?;
        let t = i.get();
        if t.value == Symbol::Colon {
            i.advance();
            let ty = self.type_expression(i)?;
            argv.push(Argument{id, ty});
        }else{
            let ty = AST::symbol(t.line,t.col,Symbol::None);
            argv.push(Argument{id, ty});
        }
        let t = i.get();
        if t.value == Symbol::Vert {
            i.advance();
            break;
        }
        self.expect(i,Symbol::Comma)?;
    }
    return Ok(argv);
}

fn lambda_expression(&mut self, t0: &Token, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let argv = self.lambda_formal_argument_list(i)?;
    let body_expression = self.expression(i)?;

    let ret_type = AST::symbol(t0.line,t0.col,Symbol::None);
    let header = Box::new(FnHeader{
        argv, id: Some(format!("{}:{}",t0.line,t0.col)),
        ret_type, type_variables: None,
        symbol_table_index: Cell::new(SYMBOL_TABLE_DANGLING)
    });
    return Ok(AST::node(t0.line, t0.col, Symbol::Function,
        Info::FnHeader(header), Some(Box::new([body_expression]))
    ));
}

fn expect(&mut self, i: &TokenIterator, value: Symbol)
-> Result<(),Error>
{
    let t = i.get();
    if t.value == value {
        i.advance();
        return Ok(());
    }else{
        return Err(syntax_error(t.line,t.col,
            format!("expected '{:?}'",value)
        ));
    }
}

fn identifier_raw(&mut self, i: &TokenIterator)
-> Result<String,Error>
{
    let t = i.get();
    if let Item::Id(ref id) = t.item {
        i.advance();
        return Ok(id.clone());
    }else{
        return Err(syntax_error(t.line,t.col,
            String::from("expected identifer.")
        ));
    }
}

fn identifier(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let t = i.get();
    if let Item::Id(ref id) = t.item {
        i.advance();
        return Ok(AST::node(t.line,t.col,Symbol::Item,
            Info::Id(id.clone()), None
        ));
    }else{
        return Err(syntax_error(t.line,t.col,
            String::from("expected identifer.")
        ));
    }
}

fn atom(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let t = i.get();
    let value = t.value;
    if value == Symbol::Item {
        i.advance();
        let info = match &t.item {
            Item::Int(x) => Info::Int(*x),
            Item::Id(s) => Info::Id(s.clone()),
            Item::String(s) => Info::String(s.clone()),
            Item::None => unreachable!()
        };
        return Ok(AST::node(t.line,t.col,Symbol::Item,info,None));
    }else if value == Symbol::True || value == Symbol::False {
        i.advance();
        return Ok(AST::symbol(t.line,t.col,value));
    }else if value == Symbol::PLeft {
        i.advance();
        let x = self.expression(i)?;
        let t = i.get();
        if t.value == Symbol::PRight {
            i.advance();
        }else{
            return Err(syntax_error(t.line,t.col,
                String::from("expected symbol ')'")
            ));
        }
        return Ok(x);
    }else if value == Symbol::BLeft {
        i.advance();
        let a = self.argument_list(i,Vec::new(),Symbol::BRight)?;
        return Ok(AST::node(t.line, t.col, Symbol::List,
            Info::None, Some(a.into_boxed_slice())
        ));
    }else if value == Symbol::Vert {
        i.advance();
        return self.lambda_expression(t,i);
    }else if value == Symbol::Null {
        i.advance();
        return Ok(AST::node(t.line, t.col, Symbol::Null,
            Info::None, None
        ));
    }else{
        return Err(syntax_error(t.line,t.col,
            format!("unexpected symbol: '{:?}'",t.value)
        ));
    }
}

fn argument_list(&mut self, i: &TokenIterator,
    mut argv: Vec<Rc<AST>>, terminator: Symbol
) -> Result<Vec<Rc<AST>>,Error>
{
    let t = i.get();
    if t.value == terminator {
        i.advance();
        return Ok(argv);
    }
    loop{
        let x = self.expression(i)?;
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
    return Ok(argv);
}

fn application(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let mut x = self.atom(i)?;
    loop{
        let t = i.get();
        if t.value == Symbol::PLeft {
            i.advance();
            let argv = self.argument_list(i,vec![x],Symbol::PRight)?;
            x = AST::node(t.line, t.col, Symbol::Application,
                Info::None, Some(argv.into_boxed_slice())
            );
        }else if t.value == Symbol::BLeft {
            i.advance();
            let argv = self.argument_list(i,vec![x],Symbol::BRight)?;
            x = AST::node(t.line, t.col, Symbol::Index,
                Info::None, Some(argv.into_boxed_slice())
            );
        }else if t.value == Symbol::Dot {
            i.advance();
            let t = i.get();
            let y = if let Item::Id(id) = &t.item {
                i.advance();
                AST::node(t.line,t.col,Symbol::Item,
                    Info::String(id.to_string()),None)
            }else{
                self.atom(i)?
            };
            x = AST::operator(t.line,t.col,Symbol::Dot,Box::new([x,y]));
        }else{
            break;
        }
    }
    return Ok(x);
}

fn power(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let x = self.application(i)?;
    let t = i.get();
    if t.value == Symbol::Pow {
        i.advance();
        let y = self.power(i)?;
        return Ok(AST::operator(t.line,t.col,t.value,Box::new([x,y])));
    }else{
        return Ok(x);
    }
}

fn negation(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let t = i.get();
    if t.value == Symbol::Minus {
        i.advance();
        let x = self.power(i)?;
        return Ok(AST::operator(t.line,t.col,Symbol::Neg,Box::new([x])));
    }else{
        return self.power(i);
    }
}

fn multiplication(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let mut x = self.negation(i)?;
    loop{
        let t = i.get();
        if t.value == Symbol::Ast  || t.value == Symbol::Div ||
           t.value == Symbol::Idiv || t.value == Symbol::Mod
        {
            i.advance();
            let y = self.negation(i)?;
            x = AST::operator(t.line,t.col,t.value,Box::new([x,y]));
        }else if t.value == Symbol::As {
            i.advance();
            let y = self.type_expression(i)?;
            x = AST::operator(t.line,t.col,t.value,Box::new([x,y]));
        }else{
            return Ok(x);
        }
    }
}

fn addition(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let mut x = self.multiplication(i)?;
    loop{
        let t = i.get();
        if t.value == Symbol::Plus || t.value == Symbol::Minus {
            i.advance();
            let y = self.multiplication(i)?;
            x = AST::operator(t.line,t.col,t.value,Box::new([x,y]));
        }else{
            return Ok(x);
        }
    }
}

fn range(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let x = self.addition(i)?;
    let t = i.get();
    if t.value == Symbol::Range {
        i.advance();
        let value = i.get().value;
        if value == Symbol::PRight || value == Symbol::BRight ||
           value == Symbol::Comma  || value == Symbol::Semicolon
        {
            return Ok(AST::operator(t.line, t.col,Symbol::Range,
               Box::new([x,
                   AST::symbol(t.line,t.col,Symbol::Null),
                   AST::symbol(t.line,t.col,Symbol::Null)])
            ));
        }else{
            let y = self.addition(i)?;
            return Ok(AST::operator(t.line, t.col,Symbol::Range,
               Box::new([x,y,AST::symbol(t.line,t.col,Symbol::Null)])
            ));
        }
    }else{
        return Ok(x);
    }
}

fn comparison(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let x = self.range(i)?;
    let t = i.get();
    let value = t.value;
    if value == Symbol::Lt || value == Symbol::Le ||
       value == Symbol::Gt || value == Symbol::Ge
    {
        i.advance();
        let y = self.range(i)?;
        return Ok(AST::operator(t.line,t.col,t.value,Box::new([x,y])));
    }else{
        return Ok(x);
    }
}

fn eq_expression(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let x = self.comparison(i)?;
    let t = i.get();
    if t.value == Symbol::Eq || t.value == Symbol::Ne {
        i.advance();
        let y = self.comparison(i)?;
        return Ok(AST::operator(t.line,t.col,t.value,Box::new([x,y])));
    }else{
        return Ok(x);
    }
}

fn conjunction(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let mut x = self.eq_expression(i)?;
    loop{
        let t = i.get();
        if t.value == Symbol::And {
            i.advance();
            let y = self.eq_expression(i)?;
            x = AST::operator(t.line,t.col,t.value,Box::new([x,y]));
        }else{
            return Ok(x);
        }
    }
}

fn disjunction(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let mut x = self.conjunction(i)?;
    loop{
        let t = i.get();
        if t.value == Symbol::Or {
            i.advance();
            let y = self.conjunction(i)?;
            x = AST::operator(t.line,t.col,t.value,Box::new([x,y]));
        }else{
            return Ok(x);
        }
    }
}

fn conditional_expression(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let x = self.disjunction(i)?;
    let t = i.get();
    if t.value == Symbol::If {
        i.advance();
        let c = self.disjunction(i)?;
        self.expect(i,Symbol::Else)?;
        let y = self.expression(i)?;
        return Ok(AST::operator(t.line, t.col, Symbol::Cond,
            Box::new([c,x,y])
        ));
    }else{
        return Ok(x);
    }
}

fn expression(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    return self.conditional_expression(i);
}

fn assignment(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let left_hand_side = self.expression(i)?;
    let t = i.get();
    if t.value == Symbol::Assignment {
        match left_hand_side.info {
            Info::Id(_) => {},
            _ => {
                return Err(syntax_error(t.line,t.col,
                    "expected an identifier on the left hand side".into()
                ));
            }
        }
        i.advance();
        let right_hand_side = self.expression(i)?;
        return Ok(AST::operator(t.line,t.col,Symbol::Assignment,
            Box::new([left_hand_side,right_hand_side])
        ));
    }else if t.value == Symbol::Semicolon {
        return Ok(AST::operator(t.line,t.col,Symbol::Statement,
            Box::new([left_hand_side])
        ));
    }else{
        return Err(syntax_error(t.line,t.col,
            format!("expected '=' or ';', got '{:?}'",t.value)
        ));
    }
}

fn formal_argument_list(&mut self, i: &TokenIterator)
-> Result<Vec<Argument>,Error>
{
    let mut argv: Vec<Argument> = Vec::new();
    let t = i.get();
    if t.value == Symbol::PRight {
        i.advance();
        return Ok(argv);
    }
    loop{
        let id = self.identifier_raw(i)?;
        self.expect(i,Symbol::Colon)?;
        let ty = self.type_expression(i)?;
        argv.push(Argument{id, ty});
        let t = i.get();
        if t.value == Symbol::PRight {
            i.advance();
            break;
        }
        self.expect(i,Symbol::Comma)?;
    }
    return Ok(argv);
}

fn type_variables(&mut self, t0: &Token, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let mut a: Vec<Rc<AST>> = Vec::new();
    loop{
        let x = self.type_atom(i)?;
        a.push(x);
        let t = i.get();
        if t.value == Symbol::Comma {
            i.advance();
        }else if t.value == Symbol::BRight {
            i.advance();
            break;
        }else{
            return Err(syntax_error(t.line,t.col,
                "expected ',' or ']'.".into()
            ));        
        }
    }
    return Ok(AST::node(t0.line, t0.col, Symbol::List,
        Info::None, Some(a.into_boxed_slice())
    ));
}

fn function_statement(&mut self, t0: &Token, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    i.advance();
    let id = self.identifier_raw(i)?;
    let t = i.get();
    let type_variables = if t.value == Symbol::BLeft {
        i.advance();
        Some(self.type_variables(t,i)?)
    }else{
        None
    };
    
    self.expect(i,Symbol::PLeft)?;
    let argv = self.formal_argument_list(i)?;

    let t = i.get();
    let ret_type = if t.value == Symbol::Colon {
        i.advance();
        self.type_expression(i)?
    }else{
        AST::symbol(t.line,t.col,Symbol::Unit)
    };
    self.expect(i,Symbol::Semicolon)?;
    let block = self.statements(i,Value::Null)?;
    self.expect(i,Symbol::End)?;

    let header = Box::new(FnHeader{
        argv, id: Some(id.clone()), ret_type, type_variables,
        symbol_table_index: Cell::new(SYMBOL_TABLE_DANGLING)
    });
    let fun = AST::node(t0.line, t0.col,
        Symbol::Function, Info::FnHeader(header), Some(Box::new([block]))
    );
    let id = identifier_from_string(id,t0.line,t0.col);
    let none = AST::symbol(t0.line,t0.col,Symbol::None);
    return Ok(AST::node(t0.line,t0.col,
        Symbol::Let, Info::None, Some(Box::new([id,none,fun]))
    ));
}

fn return_statement(&mut self, t0: &Token, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    i.advance();
    let x = self.expression(i)?;
    return Ok(AST::node(t0.line, t0.col,
        Symbol::Return, Info::None, Some(Box::new([x]))
    ));
}

fn if_statement(&mut self, t0: &Token, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    i.advance();
    let mut v: Vec<Rc<AST>> = Vec::new();
    let condition = self.expression(i)?;
    v.push(condition);
    let t = i.get();
    if t.value != Symbol::Then {
        return Err(syntax_error(t.line,t.col,
            "expected 'then'.".into()
        ));
    }
    i.advance();
    let x = self.statements(i,Value::None)?;
    v.push(x);
    loop{
        let t = i.get();
        if t.value == Symbol::End {
            i.advance();
            break;
        }else if t.value == Symbol::Else {
            i.advance();
            let x = self.statements(i,Value::None)?;
            v.push(x);
            let t = i.get();
            if t.value != Symbol::End {
                return Err(syntax_error(t.line,t.col,
                    "expected 'end'.".into()
                ));
            }
            i.advance();
            break;
        }else if t.value == Symbol::Elif {
            i.advance();
            let condition = self.expression(i)?;
            v.push(condition);
            let t = i.get();
            if t.value != Symbol::Then {
                return Err(syntax_error(t.line,t.col,
                    "expected 'then'.".into()
                ));
            }
            i.advance();
            let x = self.statements(i,Value::None)?;
            v.push(x);
        }else{
            return Err(syntax_error(t.line,t.col,
                "expected 'end' or 'else'.".into()
            ));
        }
    }
    return Ok(AST::node(t0.line, t0.col, Symbol::If, Info::None,
        Some(v.into_boxed_slice())
    ));
}

fn while_statement(&mut self, t0: &Token, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    i.advance();
    let condition = self.expression(i)?;
    let t = i.get();
    if t.value != Symbol::Do {
        return Err(syntax_error(t.line,t.col,
            "expected 'do'.".into()
        ));
    }
    i.advance();
    let body = self.statements(i,Value::None)?;
    let t = i.get();
    if t.value != Symbol::End {
        return Err(syntax_error(t.line,t.col,
            "expected 'end'.".into()
        ));
    }
    i.advance();
    return Ok(AST::node(t0.line, t0.col, Symbol::While, Info::None,
        Some(Box::new([condition,body]))
    ));
}

fn for_statement(&mut self, t0: &Token, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    i.advance();
    let lhs = self.expression(i)?;
    self.expect(i,Symbol::In)?;
    let range = self.expression(i)?;
    self.expect(i,Symbol::Do)?;
    let body = self.statements(i,Value::None)?;
    self.expect(i,Symbol::End)?;
    return Ok(AST::node(t0.line,t0.col,Symbol::For,Info::None,
        Some(Box::new([lhs,range,body]))
    ));
}

fn statements(&mut self, i: &TokenIterator, ret: Value)
-> Result<Rc<AST>,Error>
{
    let mut a: Vec<Rc<AST>> = Vec::new();
    let t0 = i.get();
    loop{
        let t = i.get();
        let line = t.line;
        let col = t.col;
        let value = t.value;
        if value == Symbol::Let {
            let mut mut_cond = Info::None;
            i.advance();
            let t = i.get();
            if t.value == Symbol::Mut {
                mut_cond = Info::Mut;
                i.advance();
            }
            let id = self.atom(i)?;

            let t = i.get();
            let texp = if t.value == Symbol::Colon {
                i.advance();
                self.type_expression(i)?
            }else{
                AST::symbol(t.line,t.col,Symbol::None)
            };

            let t = i.get();
            if t.value == Symbol::Assignment {
                i.advance();
                let x = self.expression(i)?;
                let let_exp = AST::node(line,col,Symbol::Let,mut_cond,
                    Some(Box::new([id,texp,x]))
                );
                a.push(let_exp);
                self.expect(i,Symbol::Semicolon)?;
            }else if t.value == Symbol::Semicolon {
                i.advance();
                let let_exp = AST::node(line,col,Symbol::Let,mut_cond,
                    Some(Box::new([id,texp]))
                );
                a.push(let_exp);
            }else{
                return Err(syntax_error(t.line,t.col,
                    String::from("expected '='")
                ));
            }
        }else if value == Symbol::If {
            let x = self.if_statement(t,i)?;
            a.push(x);
        }else if value == Symbol::While {
            let x = self.while_statement(t,i)?;
            a.push(x);
        }else if value == Symbol::For {
            let x = self.for_statement(t,i)?;
            a.push(x);
        }else if value == Symbol::Function {
            let x = self.function_statement(t,i)?;
            a.push(x);
        }else if value == Symbol::Terminal || value == Symbol::End ||
            value == Symbol::Else || value == Symbol::Elif
        {
            break;
        }else if value == Symbol::Return {
            let x = self.return_statement(t,i)?;
            self.expect(i,Symbol::Semicolon)?;
            a.push(x);
        }else{
            let x = self.assignment(i)?;
            a.push(x);
            self.expect(i,Symbol::Semicolon)?;
        }
    }
    if ret == Value::Null {
        a.push(AST::symbol(t0.line,t0.col,Symbol::Null));
    }
    return Ok(AST::node(t0.line, t0.col, Symbol::Block,
        Info::None, Some(a.into_boxed_slice())
    ));
}

fn type_atom(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let t = i.get();
    if let Item::Id(ref id) = t.item {
        i.advance();
        return Ok(AST::node(t.line,t.col,Symbol::Item,
            Info::Id(id.clone()), None
        ));
    }else if t.value == Symbol::PLeft {
        i.advance();
        let x = self.type_expression(i)?;
        let t = i.get();
        if t.value == Symbol::PRight {
            i.advance();
            return Ok(x);
        }else if t.value == Symbol::Comma {
            i.advance();
            let a = self.type_argument_list(vec![x],i,Symbol::PRight)?;
            return Ok(AST::node(t.line,t.col,Symbol::List,
                Info::None, Some(a.into_boxed_slice())
            ));
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

fn type_argument_list(&mut self, mut a: Vec<Rc<AST>>,
  i: &TokenIterator, terminator: Symbol
) -> Result<Vec<Rc<AST>>,Error>
{
    loop{
        let x = self.type_expression(i)?;
        a.push(x);
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
    return Ok(a);
}

fn type_application(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let x = self.type_atom(i)?;
    let t = i.get();
    if t.value == Symbol::BLeft {
        i.advance();
        let a = self.type_argument_list(vec![x],i,Symbol::BRight)?;
        return Ok(AST::node(t.line, t.col, Symbol::Application,
            Info::None, Some(a.into_boxed_slice())
        ));
    }else{
        return Ok(x);
    }
}

fn type_fn(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    let x = self.type_application(i)?;
    let t = i.get();
    if t.value == Symbol::To {
        i.advance();
        let y = self.type_fn(i)?;
        return Ok(AST::node(t.line, t.col, Symbol::Fn,
            Info::None, Some(Box::new([x,y]))
        ));
    }else{
        return Ok(x);
    }
}

fn type_expression(&mut self, i: &TokenIterator)
-> Result<Rc<AST>,Error>
{
    return self.type_fn(i);
}

// impl Parser
}

pub fn parse(s: &str) -> Result<Rc<AST>,Error> {
    let v = scan(s)?;
    // println!("{:?}\n",v);
    let i = TokenIterator::new(&v);
    let mut parser = Parser{};
    let x = parser.statements(&i,Value::None)?;
    // println!("{}",x);
    return Ok(x);
}

