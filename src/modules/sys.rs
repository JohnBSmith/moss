
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

fn cmd(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        2 => {}, n => return env.argc_error(n,2,2,"cmd")
    }
    let cmd_name = match argv[0] {
        Object::String(ref s) => s.to_string(),
        ref s => return env.type_error1(
            "Type error in cmd(command,argv): command is not a string.","command",s)
    };
    let a = match argv[1] {
        Object::List(ref a) => {
            let a = &a.borrow_mut().v;
            let mut buffer: Vec<String> = Vec::with_capacity(a.len());
            for x in a {
                let s = match x {
                    Object::String(ref s) => s.to_string(),
                    _ => return env.type_error(
                        "Type error in cmd(command,argv): argv must be a list of strings.")
                };
                buffer.push(s);
            }
            buffer
        },
        _ => panic!()
    };
    if env.rte().capabilities.borrow().command {
        match process::Command::new(&cmd_name).args(&a[..]).status() {
            Ok(status) => {
                Ok(if status.success() {Object::Null} else {Object::Int(1)})
            },
            Err(_) => env.std_exception(&format!(
                "Error in cmd(command,argv): failed to execute command=='{}'.",cmd_name))
        }
    }else{
        env.std_exception("Error in cmd(command,argv): permission denied.")
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
        m.insert_fn_plain("cmd",cmd,2,2);
    }
    return Object::Table(Rc::new(sys));
}
