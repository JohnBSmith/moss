
// Example: Calling Moss from Rust.

extern crate moss;

pub fn main(){
    let i = moss::Interpreter::new();
    let a = i.eval("
        List.map = sub|f|
            a = []
            for x in self
                a.push(f(x))
            end
            return a
        end
        a = [1,2,3,4]
        a.map(|x| 2*x)
    ");
    println!("{}",a);
}

// Output:
// [2, 4, 6, 8]

