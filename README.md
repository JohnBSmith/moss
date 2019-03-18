
[Home](https://johnbsmith.github.io/moss/home.htm)
| [Language](https://johnbsmith.github.io/moss/doc/moss/toc.htm)
| [Library](https://johnbsmith.github.io/moss/doc/library/toc.htm)
| [Examples](https://johnbsmith.github.io/moss/doc/examples/toc.htm)
| [Rust-Moss examples](doc/md/rust-moss-examples.md)

# Moss interpreter

Moss is a dynamic programming language. Its interpreter kernel
is written in Rust.

Example of calling Moss code from Rust:

```rust
use moss::object::Object;

fn main(){
    let i = moss::Interpreter::new();
    i.rte.set("a",Object::from(vec![1,2,3,4]));

    let v: Vec<i32> = i.eval_cast(r#"
        a.map(|x| 2*x)
    "#);

    println!("{:?}",v);
}
```

