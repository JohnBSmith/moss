
extern crate moss;
use std::env;
use moss::object::{Object,Map};

fn is_option(s: &str) -> bool {
  s.len()>0 && &s[0..1]=="-"
}

struct Info{
  program: Option<String>,
  file: Option<String>,
  ifile: Vec<String>,
  cmd: Option<String>
}
impl Info{
  pub fn new() -> Box<Self> {
    let mut info = Info{
      program: None,
      file: None,
      ifile: Vec::new(),
      cmd: None
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

