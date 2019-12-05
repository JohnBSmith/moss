
use std::str;
use std::env::var;
use std::io;
use std::io::Write;
use std::os::unix::io::RawFd;
use std::path::PathBuf;
use termios::{
    Termios, tcsetattr, TCSANOW, ICANON, ECHO
};

const STDIN_FILENO: i32 = 0;
const TAB: u8 = 9;
const NEWLINE: u8 = 10;
const ESC: u8 = 27;
const ARROW: u8 = 91;
const UP: u8 = 65;
const DOWN: u8 = 66;
const LEFT: u8 = 68;
const RIGHT: u8 = 67;
const BACKSPACE: u8 = 127;

use crate::object::{Object,List};

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
            if first.s==s {return;}
        }
        self.first = Some(Box::new(HistoryNode{
            s: String::from(s), next: self.first.take()
        }));
    }
}

fn getchar() -> u8 {
    let c = unsafe{libc::getchar()};
    return if c<0 {b'?'} else {c as u8};
}

fn flush() {
    io::stdout().flush().ok();
}

fn print_flush(s: &str) {
    print!("{}",s);
    io::stdout().flush().ok();
}

fn clear(cols: usize, prompt: &str, a: &[char]){
    let lines = number_of_lines(cols,prompt,&a);
    clear_input(lines);
}

fn print_prompt_flush(prompt: &str, a: &[char]) {
    print!("{}",prompt);
    for x in a {print!("{}",x);}
    flush();
}

fn number_of_bytes(c: u8) -> usize {
    let mut i: usize = 0;
    while i<=7 && c>>(7-i)&1 == 1 {
        i+=1;
    }
    return i;
}

fn number_of_lines(cols: usize, prompt: &str, a: &[char]) -> usize {
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

fn complete_u32_char(c: u8) -> char {
    if c>127 {
        let mut buffer: [u8;8] = [0,0,0,0,0,0,0,0];
        let bytes = number_of_bytes(c);
        match bytes {
            2 => {
                buffer[0] = c;
                buffer[1] = getchar();
            },
            3 => {
                buffer[0] = c;
                buffer[1] = getchar();
                buffer[2] = getchar();
            },
            4 => {
                buffer[0] = c;
                buffer[1] = getchar();
                buffer[2] = getchar();
                buffer[3] = getchar();
            },
            _ => panic!()
        };
        let s = match str::from_utf8(&buffer[0..bytes]) {
            Ok(s) => s, Err(_) => "?"
        };
        return match s.chars().next() {
            Some(x) => x, None => '?'
        };
    }else{
        return char::from(c);
    }
}


pub fn getline_history(prompt: &str, history: &History) -> io::Result<String> {
    let fd: RawFd = STDIN_FILENO;
    let tio_backup = Termios::from_fd(fd)?;
    let mut tio = tio_backup;

    tio.c_lflag &= !(ICANON|ECHO);
    tcsetattr(fd, TCSANOW, &tio)?;
    print!("{}",prompt);
    flush();

    let mut history_index=0;
    let cols = get_cols();

    let mut a0: Vec<char> = Vec::new();
    let mut a: Vec<char> = Vec::new();
    let mut i: usize = 0;
    let mut n: usize = 0;

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
                        if i>0 {i-=1; print_flush("\x1b[1D");}
                        continue;
                    }else if c3==RIGHT {
                        if i<n {i+=1; print_flush("\x1b[1C");}
                        continue;
                    }else if c3==UP {
                        if history_index == 0 {
                            a0 = a.clone();
                        }
                        if let Some(x) = history.get(history_index+1) {
                            clear(cols,prompt,&a);
                            a = x.chars().collect();
                            n = a.len(); i=n;
                            history_index+=1;
                            print_prompt_flush(prompt,&a);
                        }
                        continue;
                    }else if c3==DOWN {
                        if history_index>0 {
                            if history_index==1 {
                                clear(cols,prompt,&a);
                                history_index=0;
                                a = a0.clone();
                                n = a.len(); i=n;
                                print_prompt_flush(prompt,&a);
                            }else if let Some(x) = history.get(history_index-1) {
                                clear(cols,prompt,&a);
                                history_index-=1;
                                a = x.chars().collect();
                                n = a.len(); i=n;
                                print_prompt_flush(prompt,&a);
                            }else{
                                panic!();
                            }
                        }
                        continue;
                    }else{
                        continue;
                    }
                }else{
                    continue;
                }
            }else if c != TAB {
                continue;
            }
        }else if c==BACKSPACE {
            if i>0 {
                for j in i..n {
                    a[j-1]=a[j];
                }
                a.pop();
                n-=1; i-=1;
                print!("\x1b[D");
                for j in i..n {
                    print!("{}",a[j]);
                }
                print!(" ");
                for _ in i..n+1 {
                    print!("\x1b[D");
                }
                flush();
            }
            continue;
        }
        let cu32 = complete_u32_char(c);
        a.push('0');
        for j in (i..n).rev() {
            a[j+1]=a[j];
        }
        a[i] = cu32;
        i+=1; n+=1;
        for j in i-1..n {
            print!("{}",a[j]);
        }

        // Bug: a hanzi, say 0x4567, hampers the cursor
        // to move backward by one character.
        // (GNOME Terminal 3.18.3)
        for _ in i..n {print!("\x1b[D");}
        flush();
    }
    tcsetattr(fd, TCSANOW, &tio_backup)?;
    let s: String = a.into_iter().collect();
    return Ok(s);
}

pub fn getline(prompt: &str) -> io::Result<String> {
    let history = History{first: None};
    return getline_history(prompt,&history);
}

/*
pub fn getline(prompt: &str) -> io::Result<String> {
    print!("> ");
    io::stdout().flush().ok();
    let mut input = String::new();
    return match io::stdin().read_line(&mut input) {
        Ok(_) => Ok(input),
        Err(x) => Err(x)
    };
}
*/

pub fn init_search_paths() -> List {
    let mut a: Vec<Object> = Vec::with_capacity(2);
    a.push(Object::from("./"));
    let mut path = match var("HOME") {
        Ok(s) => PathBuf::from(s),
        _ => panic!()
    };
    path.push(".moss/");
    match path.as_path().to_str() {
        Some(s) => a.push(Object::from(s)),
        None => unreachable!()
    }
    return List{v: a, frozen: false};
}

