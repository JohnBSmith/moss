
// Example: Calling Moss from Rust.

extern crate moss;

fn main(){
    let i = moss::Interpreter::new();
    let v = i.tie(|env| {
        let a = env.eval(r#"
            List.map = fn|f|
                a = []
                for x in self
                    a.push(f(x))
                end
                return a
            end
            a = [1,2,3,4]
            a.map(|x| 2*x)
        "#);
        println!("{} (List)",a);
        env.downcast::<Vec<i32>>(&a)
    });
    println!("{:?} (Vec<i32>)",v);
}

// Output:
// [2, 4, 6, 8] (List)
// [2, 4, 6, 8] (Vec<i32>)

