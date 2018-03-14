
# Rust-Moss examples

**Table of contents**
1. [Minimal working example](#minimal-working-example)
2. [Using a return value](#using-a-return-value)
3. [Calling a Rust function from Moss](#calling-a-rust-function-from-moss)
4. [Calling a Moss function from Rust](#calling-a-moss-function-from-rust)
5. [Error handling](#error-handling)

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

## Using a return value

```rust
extern crate moss;
use moss::object::Object;

fn main(){
    let i = moss::Interpreter::new();
    let y = i.eval(r#"
        f = |n| 1 if n==0 else n*f(n-1)
        f(4)
    "#);
    let y: i32 = match y {
        Object::Int(y) => y,
        ref y => {
            i.print_type_and_value(y);
            unreachable!();
        }
    };
    println!("{}",y);
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
    i.rte.gtab.borrow_mut().insert("fac",new_i32_to_i32(fac,"fac"));
    i.eval(r#"
        print(fac(4))
    "#);
}
```

## Calling a Moss function from Rust

```rust
extern crate moss;
use moss::object::Object;
use std::rc::Rc;

trait I32TOI32 {
    fn i32_to_i32(&self, f: &str) -> Box<Fn(i32)->i32>;
}

impl I32TOI32 for Rc<moss::Interpreter> {
    fn i32_to_i32(&self, f: &str) -> Box<Fn(i32)->i32> {
        let i = self.clone();
        let fobj = i.eval(f);
        return Box::new(move |x: i32| -> i32 {
            match i.call(&fobj,&Object::Null,&[Object::Int(x)]) {
                Ok(y) => match y {
                    Object::Int(y) => y,
                    ref y => {i.print_type_and_value(y); unreachable!();}
                },
                Err(e) => {i.print_exception(&e); unreachable!();}
            }
        });
    }
}

fn main(){
    let i = Rc::new(moss::Interpreter::new());
    let fac = i.i32_to_i32("|n| (1..n).reduce(1,|x,y| x*y)");
    println!("{}",fac(4));
}
```

## Error handling

```rust
extern crate moss;
use moss::object::{Object, Map};

fn main(){
    let i = moss::Interpreter::new();
    let module_name = "";

    let gtab = Map::new();
    // table of global variables

    let y = match i.eval_string(r#"
        []+1
    "#, module_name, gtab, moss::Value::Optional) {
        Ok(y) => y,
        Err(e) => {
            panic!(format!("{}",i.exception_to_string(&e)));
        }
    };

    let value: i32 = match y {
        Object::Int(x) => x,
        x => {
            panic!(format!(
                "Type error: expected an integer, but got {}.", i.string(&x)
            ));
        }
    };

    println!("{}",value);
}
```
