
# Experimental Moss interpreter in Rust

Experimental interpreter for the dynamic programming language Moss.
This implementation is written in Rust and more type safe than
the [first](https://github.com/JohnBSmith/moss-c) implementation in C.

See [Moss Home](https://johnbsmith.github.io/moss/home.htm).

Example of calling Moss code from Rust:

```rust
extern crate moss;
use moss::object::Object;

fn main(){
    let i = moss::Interpreter::new();
    let x = i.eval("
        f = |n| 1 if n==0 else n*f(n-1)
        f(4)
    ");
    if let Object::Int(x) = x {
        println!("{}",x);
    }
}
```


