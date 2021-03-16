
use std::os::raw::{c_int, c_ulong, c_long, c_void, c_char, c_double};
use std::mem::MaybeUninit;
use std::ptr::null;
use std::rc::Rc;
use std::any::Any;

use crate::object::{
    Object, FnResult, Interface, Exception,
    float, downcast, ptr_eq_plain
};
use crate::vm::{Env, RTE};

#[allow(non_camel_case_types)]
type size_t = usize;

#[repr(C)]
struct mpz_struct {
    _mp_alloc: c_int,
    _mp_size: c_int,
    _mp_d: *mut c_void,
}

#[allow(non_camel_case_types)]
type mpz_ptr = *mut mpz_struct;

#[allow(non_camel_case_types)]
type mpz_srcptr = *const mpz_struct;

#[link(name = "gmp")]
extern "C" {
    fn __gmpz_init_set_si(rop: mpz_ptr, op: c_long);
    fn __gmpz_clear(x: mpz_ptr);
    fn __gmpz_cmp(op1: mpz_srcptr, op2: mpz_srcptr) -> c_int;
    fn __gmpz_cmp_si(op1: mpz_srcptr, op2: c_long) -> c_int;
    fn __gmpz_set (rop: mpz_ptr, op: mpz_srcptr);
    fn __gmpz_set_str (rop: mpz_ptr, s: *const c_char, base: c_int) -> c_int;

    fn __gmpz_add(rop: mpz_ptr, op1: mpz_srcptr, op2: mpz_srcptr);
    fn __gmpz_sub(rop: mpz_ptr, op1: mpz_srcptr, op2: mpz_srcptr);
    fn __gmpz_mul(rop: mpz_ptr, op1: mpz_srcptr, op2: mpz_srcptr);
    fn __gmpz_fdiv_q(q: mpz_ptr, n: mpz_srcptr, d: mpz_srcptr);
    fn __gmpz_fdiv_r(r: mpz_ptr, n: mpz_srcptr, d: mpz_srcptr);

    fn __gmpz_add_ui(rop: mpz_ptr, op1: mpz_srcptr, op2: c_ulong);
    fn __gmpz_sub_ui(rop: mpz_ptr, op1: mpz_srcptr, op2: c_ulong);
    fn __gmpz_ui_sub(rop: mpz_ptr, op1: c_ulong, op2: mpz_srcptr);
    fn __gmpz_mul_si(rop: mpz_ptr, op1: mpz_srcptr, op2: c_long);
    fn __gmpz_pow_ui(rop: mpz_ptr, base: mpz_srcptr, exp: c_ulong);
    fn __gmpz_powm(rop: mpz_ptr, base: mpz_srcptr, exp: mpz_srcptr, m: mpz_srcptr);

    fn __gmpz_get_str(s: *mut c_char, base: c_int, op: mpz_srcptr) -> *mut c_char;
    fn __gmpz_get_ui(op: mpz_srcptr) -> c_ulong;
    fn __gmpz_get_si(op: mpz_srcptr) -> c_long;
    fn __gmpz_get_d(op: mpz_srcptr) -> c_double;
    fn __gmpz_neg(rop: mpz_ptr, op: mpz_srcptr);
    fn __gmpz_abs(rop: mpz_ptr, op: mpz_srcptr);
    fn __gmpz_sizeinbase (op: mpz_srcptr, base: c_int) -> size_t;

    #[cfg(target_pointer_width = "32")]
    fn __gmpz_fits_slong_p(op: mpz_srcptr) -> c_int;

    #[cfg(target_pointer_width = "64")]
    fn __gmpz_fits_sint_p(op: mpz_srcptr) -> c_int;
}

extern "C" {
    fn strlen(cs: *const c_char) -> size_t;
    fn free(p: *mut c_void);
}

struct Mpz {
    mpz: mpz_struct
}
impl Drop for Mpz {
    fn drop(&mut self) {
        unsafe { __gmpz_clear(&mut self.mpz) }
    }
}
impl Mpz {
    fn new() -> Mpz {
        unsafe {
            let mut mpz = MaybeUninit::<mpz_struct>::uninit();
            __gmpz_init_set_si(mpz.as_mut_ptr(), 0);
            Mpz {mpz: mpz.assume_init()}
        }
    }

    fn from_int(i: c_long) -> Mpz {
        unsafe {
            let mut mpz = MaybeUninit::<mpz_struct>::uninit();
            __gmpz_init_set_si(mpz.as_mut_ptr(), i);
            Mpz {mpz: mpz.assume_init()}
        }
    }

    fn from_string(mut s: String) -> Result<Mpz,()> {
        unsafe {
            let mut mpz = Mpz::new();
            s.push('\x00');
            let base = if s.len()>3 &&
                (&s[0..2] == "0x" || &s[0..2] == "0b")
                {0} else {10};
            let p = s.as_bytes().as_ptr() as *const c_char;
            if __gmpz_set_str(&mut mpz.mpz, p, base) == 0 {
                Ok(mpz)
            } else {
                Err(())
            }
        }
    }

    fn set(&mut self, x: &Mpz) {
        unsafe {__gmpz_set(&mut self.mpz,&x.mpz);}
    }

    fn mul_int(&mut self, a: &Mpz, b: c_long) {
        unsafe {
            __gmpz_mul_si(&mut self.mpz, &a.mpz, b);
        }
    }

    fn add(&mut self, a: &Mpz, b: &Mpz) {
        unsafe {
            __gmpz_add(&mut self.mpz, &a.mpz, &b.mpz);
        }
    }

    fn sub(&mut self, a: &Mpz, b: &Mpz) {
        unsafe {
            __gmpz_sub(&mut self.mpz, &a.mpz, &b.mpz);
        }
    }

    fn mul(&mut self, a: &Mpz, b: &Mpz) {
        unsafe {
            __gmpz_mul(&mut self.mpz, &a.mpz, &b.mpz);
        }
    }

    fn add_int(&mut self, a: &Mpz, b: c_long) {
        unsafe {
              if b < 0 {
                  if b == <c_long>::min_value() {
                        panic!();
                  } else {
                        __gmpz_sub_ui(&mut self.mpz, &a.mpz, (-b) as c_ulong);
                  }
              } else {
                    __gmpz_add_ui(&mut self.mpz, &a.mpz, b as c_ulong);
              }
        }
    }

    fn sub_int(&mut self, a: &Mpz, b: c_long) {
        unsafe {
              if b<0 {
                  if b==<c_long>::min_value() {
                        panic!();
                  } else {
                        __gmpz_add_ui(&mut self.mpz, &a.mpz, (-b) as c_ulong);
                  }
              } else {
                    __gmpz_sub_ui(&mut self.mpz, &a.mpz, b as c_ulong);
              }
        }
    }

    fn int_sub(&mut self, a: c_long, b: &Mpz) {
        if a < 0 {
            if a == <c_long>::max_value() {
                panic!();
            } else {
                unsafe {
                    __gmpz_add_ui(&mut self.mpz, &b.mpz, (-a) as c_ulong);
                    __gmpz_neg(&mut self.mpz, &self.mpz);
                }
            }
        } else {
            unsafe {__gmpz_ui_sub(&mut self.mpz, a as c_ulong, &b.mpz);}
        }
    }

    fn pow_uint(&mut self, a: &Mpz, b: c_ulong) {
        unsafe {
            __gmpz_pow_ui(&mut self.mpz, &a.mpz, b);
        }
    }

    fn pow_mod(&mut self, a: &Mpz, n: &Mpz, m: &Mpz) {
        unsafe {
            __gmpz_powm(&mut self.mpz, &a.mpz, &n.mpz, &m.mpz);
        }
    }

    fn fdiv(&mut self, a: &Mpz, b: &Mpz) {
        unsafe {
            __gmpz_fdiv_q(&mut self.mpz, &a.mpz, &b.mpz);
        }
    }

    fn fdiv_int(&mut self, a: &Mpz, b: c_long) {
        let y = Mpz::from_int(b);
        unsafe {
            __gmpz_fdiv_q(&mut self.mpz, &a.mpz, &y.mpz);
        }
    }

    fn fdiv_rem(&mut self, a: &Mpz, b: &Mpz) {
        unsafe {
            __gmpz_fdiv_r(&mut self.mpz, &a.mpz, &b.mpz);
        }
    }

    fn fdiv_int_rem(&mut self, a: &Mpz, b: c_long) {
        let y = Mpz::from_int(b);
        unsafe {
            __gmpz_fdiv_r(&mut self.mpz, &a.mpz, &y.mpz);
        }
    }

    fn as_ui(&self) -> c_ulong {
        unsafe {__gmpz_get_ui(&self.mpz)}
    }

    #[cfg(target_pointer_width = "32")]
    fn try_as_si(&self) -> Result<i32,()> {
        if unsafe {__gmpz_fits_slong_p(&self.mpz)}!=0 {
            Ok(unsafe {__gmpz_get_si(&self.mpz)})
        } else {
            Err(())
        }
    }

    #[cfg(target_pointer_width = "64")]
    fn try_as_si(&self) -> Result<i32,()> {
        if unsafe {__gmpz_fits_sint_p(&self.mpz)}!=0 {
            Ok(unsafe {__gmpz_get_si(&self.mpz)} as i32)
        } else {
            Err(())
        }
    }

    fn as_f64(&self) -> f64 {
        unsafe {__gmpz_get_d(&self.mpz)}
    }

    fn to_string(&self) -> String {
        unsafe {
            let p = __gmpz_get_str(null::<i8>() as *mut i8, 10, &self.mpz);
            let len = strlen(p);
            let a: &[u8] = std::slice::from_raw_parts(p as *const u8,len);
            let s = std::str::from_utf8(a).unwrap();
            let value = s.to_string();
            free(p as *mut c_void);
            value
        }
    }

    fn to_hex(&self) -> String {
        unsafe {
            let p = __gmpz_get_str(null::<i8>() as *mut i8, 16, &self.mpz);
            let len = strlen(p);
            let a: &[u8] = std::slice::from_raw_parts(p as *const u8,len);
            let s = std::str::from_utf8(a).unwrap();
            let value = s.to_string();
            free(p as *mut c_void);
            value
        }
    }

    fn cmp(&self, b: &Mpz) -> c_int {
        unsafe {__gmpz_cmp(&self.mpz, &b.mpz)}
    }

    fn cmp_int(&self, b: i32) -> c_int {
        unsafe {__gmpz_cmp_si(&self.mpz, b.into())}
    }

    fn neg(&mut self, x: &Mpz) {
        unsafe {__gmpz_neg(&mut self.mpz, &x.mpz)}
    }
    fn abs(&mut self, x: &Mpz) {
        unsafe {__gmpz_abs(&mut self.mpz, &x.mpz)}
    }

    fn size_in_base2(&self) -> usize {
        unsafe {__gmpz_sizeinbase(&self.mpz,2)}
    }
}

impl Eq for Mpz {}
impl PartialEq for Mpz {
    fn eq(&self, other: &Mpz) -> bool {
        unsafe {__gmpz_cmp(&self.mpz, &other.mpz) == 0}
    }
}
impl PartialEq<i32> for Mpz {
    fn eq(&self, b: &i32) -> bool {
        unsafe {__gmpz_cmp_si(&self.mpz, (*b).into()) == 0}
    }
}

pub struct Long {
    value: Mpz
}

impl Long {
    #[allow(dead_code)]
    pub fn from_int(x: i32) -> Long {
        Long {value: Mpz::from_int(x.into())}
    }

    pub fn object_from_int(x: i32) -> Object {
        Object::Interface(Rc::new(Long{value: Mpz::from_int(x.into())}))
    }

    pub fn object_from_string(a: &[char]) -> Result<Object,()> {
        let s: String = a.iter().collect();
        match Mpz::from_string(s) {
            Ok(y) => {
                Ok(Object::Interface(Rc::new(Long{value: y})))
            },
            Err(()) => Err(())
        }
    }

    pub fn to_long(x: &Object) -> Result<Object,()> {
        match *x {
            Object::Int(x) => {
                Ok(Long::object_from_int(x))
            },
            Object::String(ref s) => {
                Long::object_from_string(&s.data)
            },
            Object::Interface(ref x) => {
                if x.as_any().downcast_ref::<Long>().is_some() {
                    Ok(Object::Interface(x.clone()))
                } else {
                    Err(())
                }
            },
            _ => Err(())
        }
    }

    pub fn as_f64(&self) -> f64 {
        Mpz::as_f64(&self.value)
    }
    pub fn try_as_int(&self) -> Result<i32,()> {
        Mpz::try_as_si(&self.value)
    }
    pub fn add_int_int(a: i32, b: i32) -> Object {
        let x = Mpz::from_int(a.into());
        let mut y = Mpz::new();
        y.add_int(&x,b.into());
        Object::Interface(Rc::new(Long {value: y}))
    }
    pub fn sub_int_int(a: i32, b: i32) -> Object {
        let x = Mpz::from_int(a.into());
        let mut y = Mpz::new();
        y.sub_int(&x,b.into());
        Object::Interface(Rc::new(Long {value: y}))
    }
    pub fn mul_int_int(a: i32, b: i32) -> Object {
        let x = Mpz::from_int(a.into());
        let mut y = Mpz::new();
        y.mul_int(&x,b.into());
        Object::Interface(Rc::new(Long {value: y}))
    }
    pub fn pow_int_uint(a: i32, b: u32) -> Object {
        let x = Mpz::from_int(a.into());
        let mut y = Mpz::new();
        y.pow_uint(&x,b.into());
        Object::Interface(Rc::new(Long {value: y}))
    }
    pub fn to_hex(&self) -> String {
        self.value.to_hex()
    }
}

impl Interface for Long {
    fn as_any(&self) -> &dyn Any {self}
    fn type_name(&self, _env: &mut Env) -> String {
        "Long".to_string()
    }
    fn get_type(&self, env: &mut Env) -> FnResult {
        Ok(Object::Interface(env.rte().type_long.clone()))
    }
    fn to_string(self: Rc<Self>, _env: &mut Env) -> Result<String,Box<Exception>> {
        Ok(self.value.to_string())
    }

    fn add(self: Rc<Self>, b: &Object, _env: &mut Env) -> FnResult {
        if let Object::Int(b) = *b {
            let mut y = Mpz::new();
            y.add_int(&self.value,b.into());
            Ok(Object::Interface(Rc::new(Long{value: y})))
        } else if let Some(b) = downcast::<Long>(b) {
            let mut y = Mpz::new();
            y.add(&self.value,&b.value);
            Ok(Object::Interface(Rc::new(Long{value: y})))
        } else if let Object::Float(b) = *b {
            let a = Mpz::as_f64(&self.value);
            Ok(Object::Float(a + b))
        } else {
            Ok(Object::unimplemented())
        }
    }

    fn sub(self: Rc<Self>, b: &Object, _env: &mut Env) -> FnResult {
        if let Object::Int(b) = *b {
            let mut y = Mpz::new();
            y.sub_int(&self.value,b.into());
            Ok(Object::Interface(Rc::new(Long {value: y})))
        } else if let Some(b) = downcast::<Long>(b) {
            let mut y = Mpz::new();
            y.sub(&self.value, &b.value);
            Ok(Object::Interface(Rc::new(Long {value: y})))
        } else if let Object::Float(b) = *b {
            let a = Mpz::as_f64(&self.value);
            Ok(Object::Float(a - b))
        } else {
            Ok(Object::unimplemented())
        }
    }

    fn mul(self: Rc<Self>, b: &Object, _env: &mut Env) -> FnResult {
        if let Object::Int(b) = *b {
            let mut y = Mpz::new();
            y.mul_int(&self.value,b.into());
            Ok(Object::Interface(Rc::new(Long {value: y})))
        } else if let Some(b) = downcast::<Long>(b) {
            let mut y = Mpz::new();
            y.mul(&self.value,&b.value);
            Ok(Object::Interface(Rc::new(Long {value: y})))
        } else if let Object::Float(b) = *b {
            let a = Mpz::as_f64(&self.value);
            Ok(Object::Float(a*b))
        } else {
            Ok(Object::unimplemented())
        }
    }

    fn radd(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(a) = *a {
            let mut y = Mpz::new();
            y.add_int(&self.value,a.into());
            Ok(Object::Interface(Rc::new(Long {value: y})))
        } else if let Object::Float(a) = *a {
            let b = Mpz::as_f64(&self.value);
            Ok(Object::Float(a + b))
        } else {
            env.type_error("Type error in a+b: cannot add a and b: Long.")
        }
    }

    fn rsub(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(a) = *a {
            let mut y = Mpz::new();
            y.int_sub(a.into(),&self.value);
            Ok(Object::Interface(Rc::new(Long {value: y})))
        } else if let Object::Float(a) = *a {
            let b = Mpz::as_f64(&self.value);
            Ok(Object::Float(a - b))
        } else {
            env.type_error("Type error in a-b: cannot subtract a and b: Long.")
        }
    }

    fn rmul(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(a) = *a {
            let mut y = Mpz::new();
            y.mul_int(&self.value,a.into());
            Ok(Object::Interface(Rc::new(Long {value: y})))
        } else if let Object::Float(a) = *a {
            let b = Mpz::as_f64(&self.value);
            Ok(Object::Float(a*b))
        } else {
            env.type_error("Type error in x*y: cannot multiply x and y: Long.")
        }
    }

    fn div(self: Rc<Self>, b: &Object, _env: &mut Env) -> FnResult {
        let a = Mpz::as_f64(&self.value);
        match *b {
            Object::Int(b) => return Ok(Object::Float(a/float(b))),
            Object::Float(b) => return Ok(Object::Float(a/b)),
            _ => {}
        }
        if let Some(b) = downcast::<Long>(b) {
            let b = Mpz::as_f64(&b.value);
            return Ok(Object::Float(a/b));
        }
        Ok(Object::unimplemented())
    }

    fn rdiv(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        let b = Mpz::as_f64(&self.value);
        match *a {
            Object::Int(a) => Ok(Object::Float(float(a)/b)),
            Object::Float(a) => Ok(Object::Float(a/b)),
            ref x => env.type_error1("Type error in x/y: cannot divide x by y: Long.","x",x)
        }
    }

    fn idiv(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(b) = *b {
            if b==0 {
                return env.value_error("Value error in a//b: b==0.");
            }
            let mut y = Mpz::new();
            y.fdiv_int(&self.value,b.into());
            Ok(Object::Interface(Rc::new(Long {value: y})))
        } else if let Some(b) = downcast::<Long>(b) {
            if b.value.cmp_int(0)==0 {
                return env.value_error("Value error in a//b: b==0.");
            }
            let mut y = Mpz::new();
            y.fdiv(&self.value,&b.value);
            Ok(Object::Interface(Rc::new(Long {value: y})))
        } else {
            env.type_error("Type error in a//b.")
        }
    }

    fn ridiv(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(a) = *a {
            if self.value.cmp_int(0)==0 {
                return env.value_error("Value error in a//b: b==0.");
            }
            let a = Mpz::from_int(a.into());
            let mut y = Mpz::new();
            y.fdiv(&a,&self.value);
            Ok(Object::Interface(Rc::new(Long {value: y})))
        } else {
            env.type_error("Type error in a//b.")
        }
    }

    fn imod(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(b) = *b {
            let mut y = Mpz::new();
            y.fdiv_int_rem(&self.value,b.into());
            Ok(Object::Interface(Rc::new(Long {value: y})))
        } else if let Some(b) = downcast::<Long>(b) {
            let mut y = Mpz::new();
            y.fdiv_rem(&self.value,&b.value);
            Ok(Object::Interface(Rc::new(Long{value: y})))
        } else {
            env.type_error("Type error in a%b: a: Long and b.")
        }
    }

    fn rimod(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(a) = *a {
            let a = Mpz::from_int(a.into());
            let mut y = Mpz::new();
            y.fdiv_rem(&a,&self.value);
            Ok(Object::Interface(Rc::new(Long {value: y})))
        } else {
            env.type_error("Type error in a%b: a: Long and b.")
        }
    }

    fn pow(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(b) = *b {
            if b<0 {
                return env.value_error("Value error in a^b: b<0.");
            }
            let mut y = Mpz::new();
            y.pow_uint(&self.value,(b as u32).into());
            Ok(Object::Interface(Rc::new(Long {value: y})))
        } else {
            env.type_error("Type error in a^b.")
        }
    }

    fn eq_plain(&self, b: &Object) -> bool {
        if let Object::Int(b) = *b {
            self.value == b
        } else if let Some(b) = downcast::<Long>(b) {
            self.value == b.value
        } else {
            false
        }
    }

    fn req_plain(&self, a: &Object) -> bool {
        if let Object::Int(a) = *a {
            self.value == a
        } else if let Some(a) = downcast::<Long>(a) {
            self.value == a.value
        } else {
            false
        }
    }

    fn eq(self: Rc<Self>, b: &Object, _env: &mut Env) -> FnResult {
        Ok(Object::Bool(self.eq_plain(b)))
    }

    fn req(self: Rc<Self>, a: &Object, _env: &mut Env) -> FnResult {
        Ok(Object::Bool(self.req_plain(a)))
    }

    fn lt(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(b) = *b {
            Ok(Object::Bool(self.value.cmp_int(b) < 0))
        } else if let Some(b) = downcast::<Long>(b) {
            Ok(Object::Bool(self.value.cmp(&b.value) < 0))
        } else {
            env.type_error("Type error in a<b.")
        }
    }

    fn le(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(b) = *b {
            Ok(Object::Bool(self.value.cmp_int(b) <= 0))
        } else if let Some(b) = downcast::<Long>(b) {
            Ok(Object::Bool(self.value.cmp(&b.value) <= 0))
        } else {
            env.type_error("Type error in a<=b.")
        }
    }

    fn rlt(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(a) = *a {
            Ok(Object::Bool(self.value.cmp_int(a) > 0))
        } else if let Some(a) = downcast::<Long>(a) {
            Ok(Object::Bool(self.value.cmp(&a.value) > 0))
        } else {
            env.type_error("Type error in a<b.")
        }
    }

    fn rle(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        if let Object::Int(a) = *a {
            Ok(Object::Bool(self.value.cmp_int(a) >= 0))
        } else if let Some(a) = downcast::<Long>(a) {
            Ok(Object::Bool(self.value.cmp(&a.value) >= 0))
        } else {
            env.type_error("Type error in a<b.")
        }
    }

    fn abs(self: Rc<Self>, _env: &mut Env) -> FnResult {
        let mut y = Mpz::new();
        y.abs(&self.value);
        Ok(Object::Interface(Rc::new(Long {value: y})))
    }

    fn sgn(self: Rc<Self>, _env: &mut Env) -> FnResult {
        let s = self.value.cmp_int(0);
        Ok(Object::Int(if s>0 {1} else if s<0 {-1} else {0}))
    }

    fn neg(self: Rc<Self>, _env: &mut Env) -> FnResult {
        let mut y = Mpz::new();
        y.neg(&self.value);
        Ok(Object::Interface(Rc::new(Long {value: y})))
    }

    fn is_instance_of(&self, type_obj: &Object, rte: &RTE) -> bool {
        if let Object::Interface(p) = type_obj {
            ptr_eq_plain(p, &rte.type_long)
        } else {
            false
        }
    }

    fn hash(&self) -> u64 {
        self.value.as_ui() as u64 ^ self.value.size_in_base2() as u64
    }
}

fn to_mpz(x: &Object) -> Result<Mpz,()> {
    if let Object::Int(x) = *x {
        Ok(Mpz::from_int(x.into()))
    } else if let Some(x) = downcast::<Long>(x) {
        let mut y = Mpz::new();
        y.set(&x.value);
        Ok(y)
    } else {
        Err(())
    }
}

pub fn pow_mod(env: &mut Env, a: &Object, n: &Object, m: &Object) -> FnResult {
    let a = match to_mpz(a) {
        Ok(x) => x,
        Err(()) => return env.type_error("Type error in pow(a,n,m): expected a of type Int or Long.")
    };
    let n = match to_mpz(n) {
        Ok(x) => x,
        Err(()) => return env.type_error("Type error in pow(a,n,m): expected n of type Int or Long.")
    };
    let m = match to_mpz(m) {
        Ok(x) => x,
        Err(()) => return env.type_error("Type error in pow(a,n,m): expected m of type Int or Long.")
    };
    if n.cmp_int(0) < 0 {
        return env.value_error("Value error in pow(a,n,m): n<0.");
    }
    let mut y = Mpz::new();
    y.pow_mod(&a, &n, &m);
    Ok(Object::Interface(Rc::new(Long {value: y})))
}

