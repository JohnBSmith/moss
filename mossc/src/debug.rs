
use std::rc::Rc;
use std::fmt;
use generator::{bc, CodeObject, BCSIZE, BCASIZE, BCAASIZE};

fn load_i32(a: &[u32]) -> i32 {
    a[0] as i32
}

fn load_u32(a: &[u32]) -> u32 {
    a[0]
}

fn load_u64(a: &[u32]) -> u64 {
   (a[1] as u64)<<32 | (a[0] as u64)
}

fn asm_listing(a: &[u32]) -> String {
    let mut s = String::from("Adr | Line:Col| Operation\n");
    let mut i=0;
    while i<a.len() {
        let op = a[i] as u8;
        let line = ((a[i]>>8) & 0xffff) as u16;
        let col = (a[i]>>24) as u8;
        if op != bc::FNSEP {
            let u = format!("{:04}| {:4}:{:02} | ",i,line,col);
            s.push_str(&u);
        }
        match op {
            bc::INT => {
                let x = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("push int {} (0x{:x})\n",x,x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::FLOAT => {
                let x = f64::from_bits(
                    load_u64(&a[BCSIZE+i..BCSIZE+i+2])
                );
                let u = format!("push float {:e}\n",x);
                s.push_str(&u);
                i+=BCAASIZE;
            },
            bc::IMAG => {
                let x = f64::from_bits(
                    load_u64(&a[BCSIZE+i..BCSIZE+i+2])
                );
                let u = format!("push imag {:e}\n",x);
                s.push_str(&u);
                i+=BCAASIZE;
            },
            bc::NULL => {s.push_str("null\n"); i+=BCSIZE;},
            bc::TRUE => {s.push_str("true\n"); i+=BCSIZE;},
            bc::FALSE => {s.push_str("false\n"); i+=BCSIZE;},
            bc::FNSELF => {s.push_str("function self\n"); i+=BCSIZE;},
            bc::ADD => {s.push_str("add\n"); i+=BCSIZE;},
            bc::SUB => {s.push_str("sub\n"); i+=BCSIZE;},
            bc::MUL => {s.push_str("mpy\n"); i+=BCSIZE;},
            bc::DIV => {s.push_str("div\n"); i+=BCSIZE;},
            bc::IDIV => {s.push_str("idiv\n"); i+=BCSIZE;},
            bc::MOD => {s.push_str("mod\n"); i+=BCSIZE;},
            bc::POW => {s.push_str("pow\n"); i+=BCSIZE;},
            bc::NEG => {s.push_str("neg\n"); i+=BCSIZE;},
            bc::EQ => {s.push_str("eq\n"); i+=BCSIZE;},
            bc::NE => {s.push_str("not eq\n"); i+=BCSIZE;},
            bc::LT => {s.push_str("lt\n"); i+=BCSIZE;},
            bc::GT => {s.push_str("gt\n"); i+=BCSIZE;},
            bc::LE => {s.push_str("le\n"); i+=BCSIZE;},
            bc::GE => {s.push_str("not ge\n"); i+=BCSIZE;},
            bc::IS => {s.push_str("is\n"); i+=BCSIZE;},
            bc::OF => {s.push_str("of\n"); i+=BCSIZE;},
            bc::ISNOT => {s.push_str("is not\n"); i+=BCSIZE;},
            bc::IN => {s.push_str("in\n"); i+=BCSIZE;},
            bc::NOTIN => {s.push_str("not in\n"); i+=BCSIZE;},
            bc::NOT => {s.push_str("not\n"); i+=BCSIZE;},
            bc::RANGE => {s.push_str("range\n"); i+=BCSIZE;},
            bc::TABLE => {s.push_str("table\n"); i+=BCSIZE;},
            bc::LIST => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("list, size={}\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::MAP => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("map, size={}\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::LOAD => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("load global [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::LOAD_ARG => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("load argument [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::LOAD_LOCAL => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("load local [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::LOAD_CONTEXT => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("load context [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::STORE => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("store global [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::STORE_ARG => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("store argument [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::STORE_LOCAL => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("store local [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::STORE_CONTEXT => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("store context [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::STR => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("string literal [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::LONG => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("long literal [{}]\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::AND => {
                let x = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("and {}\n",i as i32+x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::OR => {
                let x = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("or {}\n",i as i32+x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::ELSE => {
                let x = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("else {}\n",i as i32+x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::JMP => {
                let x = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);

                // Resolve position independent code
                // to make the listing human readable.
                let u = format!("jmp {}\n",i as i32+x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::JZ => {
                let x = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("jz {}\n",i as i32+x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::JNZ => {
                let x = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("jnz {}\n",i as i32+x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::NEXT => {
                let x = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("next {}\n",i as i32+x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::GET => {
                let x = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("get {}\n",x);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::CALL => {
                let argc = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                let u = format!("call, argc={}\n",argc);
                s.push_str(&u);
                i+=BCASIZE;
            },
            bc::RET => {s.push_str("ret\n"); i+=BCSIZE;},
            bc::YIELD => {s.push_str("yield\n"); i+=BCSIZE;},
            bc::RAISE => {s.push_str("raise\n"); i+=BCSIZE;},
            bc::FNSEP => {s.push_str("\nFunction\n"); i+=BCSIZE;},
            bc::FN => {
                let address = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);
                let argc_min = load_i32(&a[BCSIZE+i+1..BCSIZE+i+2]);
                let argc_max = load_i32(&a[BCSIZE+i+2..BCSIZE+i+3]);
                // let var_count = load_i32(&a[BCSIZE+i+3..BCSIZE+i+4]);

                // Resolve position independent code
                // to make the listing human readable.
                let u = format!("fn [{}], argc_min={}, argc_max={}\n",
                    i as i32+address, argc_min, argc_max
                );
                s.push_str(&u);
                i+=BCSIZE+4;
            },
            bc::GET_INDEX => {
                let argc = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                if argc>1 {
                    s.push_str(&format!("get index ({} args)\n",argc));
                }else{
                    s.push_str("get index\n");
                }
                i+=BCASIZE;
            },
            bc::SET_INDEX => {
                let argc = load_u32(&a[BCSIZE+i..BCSIZE+i+1]);
                if argc>1 {
                    s.push_str(&format!("set index ({} args)\n",argc));
                }else{
                    s.push_str("set index\n");
                }
                i+=BCASIZE;
            },
            bc::DOT => {s.push_str("dot\n"); i+=BCSIZE;},
            bc::DOT_SET => {s.push_str("dot set\n"); i+=BCSIZE;},
            bc::SWAP => {s.push_str("swap\n"); i+=BCSIZE;},
            bc::DUP => {s.push_str("dup\n"); i+=BCSIZE;},
            bc::DUP_DOT_SWAP => {s.push_str("dup dot swap\n"); i+=BCSIZE;},
            bc::POP => {s.push_str("pop\n"); i+=BCSIZE;},
            bc::EMPTY => {s.push_str("empty\n"); i+=BCSIZE;},
            bc::AOP => {s.push_str("aop\n"); i+=BCSIZE;},
            bc::AOP_INDEX => {s.push_str("aop index\n"); i+=BCSIZE;},
            bc::OP => {
                s.push_str("op ");
                i+=BCSIZE;
                let op = a[i] as u8;
                if op==bc::TRY {
                    let x = load_i32(&a[BCSIZE+i..BCSIZE+i+1]);
                    let u = format!("try, catch {}\n",i as i32+x);
                    s.push_str(&u);
                    i+=BCASIZE;
                }else if op==bc::TRYEND {
                    s.push_str("try end\n");
                    i+=BCSIZE;
                }else if op==bc::GETEXC {
                    s.push_str("get exception\n");
                    i+=BCSIZE;
                }else if op==bc::CRAISE {
                    s.push_str("raise further\n");
                    i+=BCSIZE;
                }else{
                    unreachable!("op ??");
                }
            },
            bc::APPLY => {s.push_str("apply\n"); i+=BCSIZE;}
            bc::HALT => {s.push_str("halt\n"); i+=BCSIZE;},
            _ => {unreachable!();}
        }
    }
    return s;
}

fn print_data(a: &[Rc<str>], f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f,"Data")?;
    for i in 0..a.len() {
        writeln!(f,"[{}]: \"{}\"",i,a[i])?;
    }
    if a.len()==0 {
        writeln!(f,"empty\n")?;
    }else{
        writeln!(f,"")?;
    }
    return Ok(());
}

impl fmt::Display for CodeObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n", asm_listing(&self.program))?;
        print_data(&self.data,f)
    }
}



