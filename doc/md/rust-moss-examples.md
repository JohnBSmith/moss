
# Rust-Moss examples

**Table of contents**
1. [Minimal working example](#minimal-working-example)
2. [Simple data interfaces](#simple-data-interfaces)
3. [String interfaces](#string-interfaces)
4. [Calling a Rust function from Moss](#calling-a-rust-function-from-moss)
5. [Calling a Moss function from Rust](#calling-a-moss-function-from-rust)
6. [Error handling](#error-handling)

## Minimal working example

```rust
extern crate moss;

fn main(){
    let i = moss::Interpreter::new();
    i.eval(r#"
        print("Hello, world!")
    "#);
}
```

## Simple data interfaces

```rust
extern crate moss;
use moss::object::Object;

fn main(){
    let i = moss::Interpreter::new();
    i.rte.set("a",Object::from(1));
    i.rte.set("b",Object::from(2));
    let y = i.eval_cast::<i32>("a+b");
    println!("{}",y);
}
```

## String interfaces

```rust
extern crate moss;
use moss::object::Object;

fn main(){
    let i = moss::Interpreter::new();
    i.rte.set("s",Object::from("[2,4,5,1,3]"));
    let s = i.eval_cast::<String>(r#"
        str(eval(s).sort())
    "#);
    println!("{}",s);
}
```

As a function:

```rust
extern crate moss;
use moss::object::{Object,Env};

fn call_str(env: &mut Env, f: &Object, s: &str) -> String {
    let y = env.call(&f,&Object::Null,&[Object::from(s)]);
    let y = env.expect_ok(y);
    return env.downcast::<String>(&y);
}

fn main(){
    let i = moss::Interpreter::new();
    let s = i.tie(|env| {
        let f = env.eval("|s| str(eval(s).sort())");
        call_str(env,&f,"[2,4,5,1,3]")
    });
    println!("{}",s);
}
```

## Calling a Rust function from Moss

```rust
extern crate moss;
use moss::object::{Object,Function,FnResult,Env};

fn new_i32_to_i32(f: fn(i32)->i32, id: &str) -> Object {
    let err = format!("Type error in {}(n): n is not an intger.",id);
    let fp = move |env: &mut Env, _pself: &Object, argv: &[Object]| -> FnResult {
        match argv[0] {
            Object::Int(n) => Ok(Object::Int(f(n))),
            ref n => env.type_error1(&err,"n",&n)
        }
    };
    return Function::mutable(Box::new(fp),1,1);
}

fn fac(n: i32) -> i32 {
    if n==0 {1} else {n*fac(n-1)}
}

fn main(){
    let i = moss::Interpreter::new();
    i.rte.set("fac",new_i32_to_i32(fac,"fac"));
    i.eval(r#"
        print(fac(4))
    "#);
}
```

In more general terms:

```rust
extern crate moss;
use moss::object::{Object,Function,FnResult,Env,Downcast,TypeName};

trait FnObj<X,Y> {
    fn new(self, f: fn(X)->Y) -> Object;
}

impl<'a,X,Y> FnObj<X,Y> for &'a str
where 
    X: TypeName, X: Downcast<Output=X>, Object: From<Y>,
    X: 'static, Y: 'static
{
    fn new(self, f: fn(X)->Y) -> Object {
        let err = format!("Type error in {}(x): x is not of type {}.",self,X::type_name());
        let fp = move |env: &mut Env, _pself: &Object, argv: &[Object]| -> FnResult {
            match X::try_downcast(&argv[0]) {
                Some(n) => Ok(Object::from(f(n))),
                None => env.type_error1(&err,"x",&argv[0])
            }
        };
        return Function::mutable(Box::new(fp),1,1);
    }
}

fn fac(n: i32) -> i32 {
    if n==0 {1} else {n*fac(n-1)}
}

fn main(){
    let i = moss::Interpreter::new();
    i.rte.set("fac",FnObj::new("fac",fac));
    i.eval(r#"
        print(fac(4))
    "#);
}
```

## Calling a Moss function from Rust

```rust
extern crate moss;
use moss::object::{Object,Downcast,TypeName};
use std::rc::Rc;

trait Function<X,Y> {
    fn new(&self, f: &str) -> Box<Fn(X)->Y>;
}

impl<X,Y> Function<X,Y> for Rc<moss::Interpreter>
    where Y: Downcast<Output=Y>+TypeName, Object: From<X>
{
    fn new(&self, f: &str) -> Box<Fn(X)->Y> {
        let i = self.clone();
        let fobj = i.eval(f);
        return Box::new(move |x: X| -> Y {
            i.tie(|env| {
                let y = env.call(&fobj,&Object::Null,&[Object::from(x)]);
                let y = env.expect_ok(y);
                env.downcast::<Y>(&y)
            })
        });
    }
}

fn main(){
    let i = moss::Interpreter::new();
    let fac = Function::<i32,i32>::new(&i,r#"
        |n| (1..n).reduce(1,|x,y| x*y)
    "#);
    println!("{}",fac(4));
}
```

## Error handling

```rust
extern crate moss;
use moss::object::{Object, Map, Env};
use moss::Value::Optional;

fn proc1(env: &mut Env) -> Result<(),String> {
    let gtab = Map::new(); // table of global variables
    let module_name = "proc1 module";
    let src = "[]+1";
    let y = env.eval_string(src, module_name, gtab, Optional);
    let y = env.map_err_string(y)?;

    let value: i32 = match y {
        Object::Int(x) => x,
        _ => return Err(format!(
            "Type error in line {}: not an integer.",line!()))
    };

    println!("{}",value);
    return Ok(());
}

fn main(){
    let i = moss::Interpreter::new();
    if let Err(e) = i.tie(proc1) {
        println!("{}",e);
    }
}
```
