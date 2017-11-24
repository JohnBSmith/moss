
extern crate moss;
use std::env;
use moss::object::{Object,Map};

const HELP: &'static str = r#"
Usage: moss [options] [file] [arguments]

Options:
-i file     Include and execute a file before normal execution.
            Multiple files are possible: '-i file1 -i file2'.
-m          Math mode: use moss as a calculator.
-e "1+2"    Evaluate some Moss code inline.
"#;

fn is_option(s: &str) -> bool {
  s.len()>0 && &s[0..1]=="-"
}

struct Info{
  program: Option<String>,
  file: Option<String>,
  ifile: Vec<String>,
  cmd: Option<String>,
  exit: bool,
}
impl Info{
  pub fn new() -> Box<Self> {
    let mut info = Info{
      program: None,
      file: None,
      ifile: Vec::new(),
      cmd: None,
      exit: false
    };
    let mut first = true;
    let mut ifile = false;
    let mut cmd = false;
    for s in env::args() {
      if first {
        info.program = Some(s);
        first = false;
      }else if is_option(&s) {
        if s=="-i" {
          ifile = true;
        }else if s=="-e" {
          cmd = true;
        }else if s=="-h" || s=="-help" || s=="--help" {
          println!("{}",HELP);
          info.exit = true;
          return Box::new(info);
        }else{
          println!("Error: unknown option: {}.",&s);
        }
      }else if ifile {
        info.ifile.push(s);
        ifile = false;
      }else if cmd {
        info.cmd = Some(s);
      }else{
        info.file = Some(s);
      }
    }
    return Box::new(info);
  }
}

fn main(){
  let i = moss::Interpreter::new();
  let gtab = Map::new();
  let info = Info::new();
  if info.exit {return;}
  for file in &info.ifile {
    i.eval_file(file,gtab.clone());
  }
  if let Some(ref id) = info.file {
    i.eval_file(id,gtab);
  }else if let Some(ref cmd) = info.cmd {
    let x = i.eval_env(cmd,gtab);
    if x != Object::Null {
      println!("{}",x);
    }
  }else{
    i.command_line_session(gtab);
  }
}

