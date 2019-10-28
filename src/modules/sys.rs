
use std::rc::Rc;
use std::any::Any;
use std::process;

use crate::object::{
    Object, FnResult, Interface, Exception,
    VARIADIC, new_module, downcast
};
use crate::vm::{RTE,Env};
use crate::class::{Class,Table};

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

fn eput(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    for i in 0..argv.len() {
        eprint!("{}",argv[i].string(env)?);
    }
    return Ok(Object::Null);
}

fn eprint(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    for i in 0..argv.len() {
        eprint!("{}",argv[i].string(env)?);
    }
    eprintln!();
    return Ok(Object::Null);
}

fn istable(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"istable")
    }
    return Ok(Object::Bool(match downcast::<Table>(&argv[0]) {
        Some(_) => true, None => false
    }));
}

fn isclass(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"isclass")
    }
    return Ok(Object::Bool(match downcast::<Class>(&argv[0]) {
        Some(_) => true, None => false
    }));
}

struct Id{
    value: u64
}

impl Interface for Id {
    fn as_any(&self) -> &dyn Any {self}
    fn to_string(self: Rc<Self>, _env: &mut Env) -> Result<String,Box<Exception>> {
        Ok(String::from(&format!("0x{:x}",self.value)))
    }
    fn hash(&self) -> u64 {self.value}
    fn eq_plain(&self, b: &Object) -> bool {
        if let Some(b) = downcast::<Id>(b) {
            self.value == b.value
        }else{
            false
        }
    }
    fn req_plain(&self, a: &Object) -> bool {
        self.eq_plain(a)
    }
    fn eq(self: Rc<Self>, b: &Object, _env: &mut Env) -> FnResult {
        Ok(Object::Bool(self.eq_plain(b)))
    }
    fn req(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        self.eq(a,env)
    }
}

fn ptr_as_u64<T: ?Sized>(p: &Rc<T>) -> u64 {
    return &**p as *const T as *const () as u64;
}

fn address(x: u64) -> Object {
    Object::Interface(Rc::new(Id{value: x}))
}

fn id(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"id")
    }
    Ok(match &argv[0] {
        Object::String(x) => address(ptr_as_u64(x)),
        Object::List(x) => address(ptr_as_u64(x)),
        Object::Map(x) => address(ptr_as_u64(x)),
        Object::Function(x) => address(ptr_as_u64(x)),
        Object::Interface(x) => address(ptr_as_u64(x)),
        _ => Object::Null
    })
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
        m.insert_fn_plain("call",crate::vm::sys_call,2,VARIADIC);
        m.insert_fn_plain("cmd",cmd,2,2);
        m.insert_fn_plain("eput",eput,0,VARIADIC);
        m.insert_fn_plain("eprint",eprint,0,VARIADIC);
        m.insert_fn_plain("istable",istable,1,1);
        m.insert_fn_plain("isclass",isclass,1,1);
        m.insert_fn_plain("id",id,1,1);
    }
    return Object::Interface(Rc::new(sys));
}
