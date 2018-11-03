
use std::rc::Rc;
use std::process;
use object::{Object, FnResult, VARIADIC, new_module};
use vm::{RTE,Env};

fn exit(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        0 => {
            process::exit(0);
        },
        1 => {
            let x = match argv[0] {
                Object::Int(x)=>x,
                ref x => return env.type_error1(
                    "Type error in exit(n): n is not an integer.",
                    "n", x
                )
            };
            process::exit(x);
        },
        n => {
            env.argc_error(n,0,1,"exit")
        }
    }
}

pub fn load_sys(rte: &Rc<RTE>) -> Object {
    let sys = new_module("sys");
    {
        let mut m = sys.map.borrow_mut();
        if let Some(ref argv) = *rte.argv.borrow() {
            m.insert("argv", Object::List(argv.clone()));
        }
        m.insert("path",Object::List(rte.path.clone()));
        m.insert_fn_plain("exit",exit,0,1);
        m.insert_fn_plain("call",::vm::sys_call,2,VARIADIC);
    }
    return Object::Table(Rc::new(sys));
}
