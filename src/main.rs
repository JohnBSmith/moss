
extern crate moss;
use std::env;
use moss::object::{Object,Map};
use moss::CompilerExtra;

const HELP: &'static str = r#"
Usage: moss [options] [file] [arguments]

Options:
-i file     Include and execute a file before normal execution.
            Multiple files are possible: '-i file1 -i file2'.

-m          Math mode: use moss as a calculator.

-e "1+2"    Evaluate some Moss code inline.

-d          Debug mode: compile assert statements.
"#;

const MATH: &'static str = r#"
use math{
  e, pi, nan, inf,
  floor, ceil, exp, sqrt, ln, lg,
  sin, cos, tan, sinh, cosh, tanh,
  asin, acos, atan, asinh, acosh, atanh,
  hypot, atan2, gamma, erf
}
"#;

fn is_option(s: &str) -> bool {
    s.len()>0 && &s[0..1]=="-"
}

struct IFile{
    s: String,
    e: bool
}

struct Info{
    program: Option<String>,
    ifile: Vec<IFile>,
    argv: Vec<String>,
    cmd: Option<String>,
    exit: bool,
    math: bool,
    debug_mode: bool,
    unsafe_mode: bool
}

impl Info{
    pub fn new() -> Box<Self> {
        let mut info = Info{
            program: None,
            ifile: Vec::new(),
            argv: Vec::new(),
            cmd: None,
            exit: false,
            math: false,
            debug_mode: false,
            unsafe_mode: false
        };
        let mut first = true;
        let mut ifile = false;
        let mut cmd = false;
        let mut args = false;
        for s in env::args() {
            if args {
                info.argv.push(s);
            }else if first {
                info.program = Some(s);
                first = false;
            }else if is_option(&s) {
                if s=="-i" {
                    ifile = true;
                }else if s=="-e" {
                    cmd = true;
                }else if s=="-ei" {
                    ifile = true;
                    cmd = true;
                }else if s=="-m" {
                    info.math = true;
                }else if s=="-h" || s=="-help" || s=="--help" {
                    println!("{}",HELP);
                    info.exit = true;
                    return Box::new(info);
                }else if s=="-d" {
                    info.debug_mode = true;
                }else if s=="-unsafe" {
                    info.unsafe_mode = true;
                }else{
                    println!("Error: unknown option: {}.",&s);
                }
            }else if ifile {
                info.ifile.push(IFile{s: s, e: cmd});
                ifile = false;
                cmd = false;
            }else if cmd {
                info.cmd = Some(s);
            }else{
                info.argv.push(s);
                args = true;
            }
        }
        return Box::new(info);
    }
}

fn main(){
    let info = Info::new();
    let i = moss::Interpreter::new();
    i.set_config(CompilerExtra{
        debug_mode: info.debug_mode
    });
    i.set_capabilities(info.unsafe_mode);

    let gtab = Map::new();
    i.rte.clear_at_exit(gtab.clone());

    if info.exit {return;}

    {
        let mut argv = i.rte.argv.borrow_mut();
        *argv = Some(moss::new_list_str(&info.argv));
    }
    let mut ilock = i.lock();
    let mut env = ilock.env();

    if info.math {
        env.eval_env(MATH,gtab.clone());
    }
    for file in &info.ifile {
        if file.e {
            env.eval_env(&file.s,gtab.clone());
        }else{
            env.eval_file(&file.s,gtab.clone());
        }
    }
    if let Some(ref id) = info.argv.first() {
        if id.len()==0 {
            env.command_line_session(gtab);
        }else{
            env.eval_file(id,gtab);
        }
    }else if let Some(ref cmd) = info.cmd {
        let x = env.eval_env(cmd,gtab);
        if x != Object::Null {
            match x.repr(&mut env) {
                Ok(s) => println!("{}",s),
                Err(e) => {
                    println!("{}",env.exception_to_string(&e));
                    println!("[exception in Interpreter::repr, see stdout]");
                }
            }
        }
    }else{
        env.command_line_session(gtab);
    }
}

