
extern crate libc;
extern crate termios;
use std::str;
use std::io;
use std::io::Write;
use std::os::unix::io::RawFd;
use self::termios::{
  Termios, tcsetattr, TCSANOW, ICANON, ECHO
};
const STDIN_FILENO: i32 = 0;
const NEWLINE: i32 = 10;
const ESC: i32 = 27;
const ARROW: i32 = 91;
const UP: i32 = 65;
const DOWN: i32 = 66;
const LEFT: i32 = 68;
const RIGHT: i32 = 67;
const BACKSPACE: i32 = 127;

fn get_win_size() -> (usize, usize) {
  use std::mem::zeroed;
  unsafe {
    let mut size: libc::winsize = zeroed();
    match libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut size) {
      0 => (size.ws_col as usize, size.ws_row as usize),
      _ => (80, 24),
    }
  }
}

fn get_cols() -> usize {
  let (cols,_) = get_win_size();
  return cols;
}

struct HistoryNode{
  s: String,
  next: Option<Box<HistoryNode>>
}
pub struct History{
  first: Option<Box<HistoryNode>>
}

impl History{
  pub fn get(&self, index: usize) -> Option<String> {
    if index==0 {return None;}
    if let Some(ref first) = self.first {
      let mut p = first;
      for _ in 0..index-1 {
        if let Some(ref next) = p.next {
          p = next;
        }else{
          return None;
        }
      }
      return Some(p.s.clone());
    }
    return None;
  }
  pub fn new() -> History{
    return History{first: None};
  }
  pub fn append(&mut self, s: &str){
    if let Some(ref first) = self.first {
      if &first.s==s {return;}
    }
    self.first = Some(Box::new(HistoryNode{
      s: String::from(s), next: self.first.take()
    }));
  }
}

fn getchar() -> i32 {
  unsafe{libc::getchar()}
}

fn number_of_bytes(c: u8) -> usize {
  let mut x=c;
  let mut i: i32 =7;
  while i>=0 && x>>i&1 == 1 {
    i-=1;
  }
  return 7-i as usize;
}

fn number_of_lines(cols: usize, prompt: &str, a: &Vec<char>) -> usize{
  let n = prompt.len()+a.len();
  return if n==0 {1} else {(n-1)/cols+1};
}

fn clear_input(lines: usize){
  print!("\x1b[2K\r");
  for _ in 1..lines {
    print!("\x1b[1A");
    print!("\x1b[2K\r");
  }
}

pub fn getline_history(prompt: &str, history: &History) -> io::Result<String> {
  let fd: RawFd = STDIN_FILENO;
  let tio_backup = try!(Termios::from_fd(fd));
  let mut tio = tio_backup.clone();

  tio.c_lflag &= !(ICANON|ECHO);
  try!(tcsetattr(fd, TCSANOW, &tio));
  let mut a: Vec<char> = Vec::new();
  let mut history_index=0;
  print!("{}",prompt);
  io::stdout().flush().ok();
  let mut i=0;
  let mut n=0;
  loop{
    let c = getchar();
    if c==NEWLINE {
      println!();
      break;
    }else if c<32 {
      if c==ESC {
        let c2 = getchar();
        if c2==ARROW {
          let c3 = getchar();
          if c3==LEFT {
            if i>0 {i-=1;}
          }else if c3==RIGHT {
            if i<n {i+=1;}
          }else if c3==UP {
            if let Some(x) = history.get(history_index+1) {
              a = x.chars().collect();
              n = a.len(); i=n;
              history_index+=1;
            }
          }else if c3==DOWN {
            if history_index>0 {
              if history_index==1 {
                history_index=0;
                a = vec![];
                i=0; n=0;                  
              }else if let Some(x) = history.get(history_index-1) {
                history_index-=1;
                a = x.chars().collect();
                n = a.len(); i=n;
              }else{
                panic!();
              }
            }
          }
        }
      }else{
        continue;
      }
    }else if c==BACKSPACE {
      if i>0 {
        for mut j in i..n {
          a[j-1]=a[j];
        }
        a.pop();
        n-=1; i-=1;
      }
    }else{
      let cu32 = if c>127 {
        let mut buffer: [u8;8] = [0,0,0,0,0,0,0,0];
        let bytes = number_of_bytes(c as u8);
        match bytes {
          2 => {
            buffer[0]=c as u8;
            buffer[1]=getchar() as u8;
          },
          3 => {
            buffer[0]=c as u8;
            buffer[1]=getchar() as u8;
            buffer[2]=getchar() as u8;
          },
          4 => {
            buffer[0]=c as u8;
            buffer[1]=getchar() as u8;
            buffer[2]=getchar() as u8;
            buffer[3]=getchar() as u8;          
          },
          _ => panic!()
        };
        let s = match str::from_utf8(&buffer[0..bytes]) {
          Ok(s) => s, Err(_) => "?"
        };
        match s.chars().next() {
          Some(x) => x, None => '?'
        }
      }else{
        c as u8 as char
      };
      a.push('0');
      for mut j in (i..n).rev() {
        a[j+1]=a[j];
      }
      a[i] = cu32;
      n+=1; i+=1;
    }
    let cols = get_cols();
    let lines = number_of_lines(cols,prompt,&a);
    clear_input(lines);
    print!("{}",prompt);
    for x in &a {print!("{}",x);}

    // Bug: a hanzi, say 0x4567, hampers the cursor
    // to move backward by one character.
    // (GNOME Terminal 3.18.3)
    for _ in i..n {print!("\x1b[D");}
    io::stdout().flush().ok();
  }

  try!(tcsetattr(fd, TCSANOW, &tio_backup));
  let s: String = a.into_iter().collect();
  return Ok(s);
}

pub fn getline(prompt: &str) -> io::Result<String>{
  let history = History{first: None};
  return getline_history(prompt,&history);
}

/*
pub fn getline(prompt: &str) -> io::Result<String>{
  print!("> ");
  io::stdout().flush().ok();
  let mut input = String::new();
  return match io::stdin().read_line(&mut input) {
    Ok(_) => Ok(input),
    Err(x) => Err(x)
  };
}
*/
