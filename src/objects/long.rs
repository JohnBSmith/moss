
#![allow(dead_code)]

use std::cmp::{min};
use std::fmt;

type Digit = u8;
const HEX_COUNT: usize=2;

struct Long{
    v: Vec<Digit>,
    minus: bool
}
impl fmt::LowerHex for Long {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &x in self.v[..].iter().rev() {
            try!(write!(f, "{:01$x}", x, HEX_COUNT));
        }
        Ok(())
    }
}

fn add(y: &mut Long, a: &[Digit], b: &[Digit]){
    let mut carry: Digit = 0;
    for (&dx,&dy) in a.iter().zip(b) {
        let (sum,overflow) = dx.overflowing_add(dy);
        if overflow {
            y.v.push(sum+carry);
            carry=1;
        }else{
            if sum==Digit::max_value() {
                y.v.push(sum.wrapping_add(carry));
            }else{
                y.v.push(sum+carry);
                carry=0;
            }
        }
    }
    let alen = a.len();
    let blen = b.len();
    let n = min(alen,blen);
    let x = if alen<blen {b} else {a};
    for &dx in x[n..].iter() {
        if dx==Digit::max_value() {
            y.v.push(dx.wrapping_add(carry));
        }else{
            y.v.push(dx);
            carry=0;
        }
    }
    if carry==1 {
        y.v.push(carry);
    }
}

fn main(){
    let x = Long{v: vec![0,0,0,255,1], minus: false};
    let y = Long{v: vec![0,0,0,255,1], minus: false};
    let mut z = Long{v: Vec::new(), minus: false};
    add(&mut z,&x.v,&y.v);
    println!("0x{:x}\n0x{:x}\n0x{:x}",x,y,z);
}
