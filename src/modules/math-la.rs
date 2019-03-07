
// Linear algebra

use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;

use crate::object::{
    Object, FnResult, List, Table, Interface,
    Exception, new_module, downcast, VARIADIC
};
use crate::vm::{
    Env, op_neg, op_add, op_sub, op_mul, op_div, op_eq,
    interface_types_set, interface_index
};
use crate::complex::Complex64;

struct ShapeStride{
    shape: usize,
    stride: isize
}

struct Array{
    n: usize,
    base: usize,
    s: Box<[ShapeStride]>,
    data: Rc<RefCell<Vec<Object>>>
}

impl Array {
    fn vector(a: Vec<Object>) -> Rc<Array> {
        Rc::new(Array{
            s: Box::new([ShapeStride{shape: a.len(), stride: 1}]),
            n: 1, base: 0, data: Rc::new(RefCell::new(a))
        })
    }
    fn matrix(m: usize, n: usize, a: Vec<Object>) -> Rc<Array> {
        Rc::new(Array{
            s: Box::new([
                ShapeStride{shape: m, stride: n as isize},
                ShapeStride{shape: n, stride: 1}
            ]),
            n: 2, base: 0, data: Rc::new(RefCell::new(a)),
        })
    }
}

impl Interface for Array {
    fn as_any(&self) -> &dyn Any {self}
    fn type_name(&self, _env: &mut Env) -> String {
        "Array".to_string()
    }
    fn to_string(self: Rc<Self>, env: &mut Env) -> Result<String,Box<Exception>> {
        if self.n==1 {
            let mut s = "vector(".to_string();
            let mut first = true;
            let stride = self.s[0].stride;
            let base = self.base as isize;
            let data = self.data.borrow();
            for i in 0..self.s[0].shape {
                if first {first = false;}
                else {s.push_str(", ");}
                let x = &data[(base+i as isize*stride) as usize];
                s.push_str(&x.repr(env)?);
            }
            s.push_str(")");
            return Ok(s);
        }else if self.n==2 {
            let m = self.s[0].shape;
            let n = self.s[1].shape;
            let mut s = "matrix(\n".to_string();
            let mut first = true;
            let base = self.base as isize;
            let istride = self.s[0].stride;
            let jstride = self.s[1].stride;
            let data = self.data.borrow();
            for i in 0..m {
                if first {first = false;}
                else {s.push_str(",\n");}
                let ibase = base+i as isize*istride;
                s.push_str("  [");
                let mut jfirst = true;
                
                for j in 0..n {
                    if jfirst {jfirst = false;}
                    else {s.push_str(", ");}
                    let x = &data[(ibase+j as isize*jstride) as usize];
                    s.push_str(&x.repr(env)?);
                }
                s.push_str("]");
            }
            s.push_str("\n)");
            return Ok(s);
        }else{
            panic!();
        }
    }
    fn get(&self, key: &Object, env: &mut Env) -> FnResult {
        if let Object::String(ref s) = *key {
            let v = &s.data;
            match v.len() {
                1 => {
                    if v[0] == 'T' {
                        return Ok(Object::Interface(transpose(self)));
                    }else if v[0] == 'H' {
                        return Ok(Object::Interface(transpose(&conj(self))));
                    }
                },
                2 => if v[0..2] == ['t','r'] {
                    return trace(env,self);
                },
                3 => {
                    if v[0..3] == ['a','b','s'] {
                        return abs(env,self);
                    }
                },
                4 => {
                    if v[0..4] == ['c','o','n','j'] {
                        return Ok(Object::Interface(conj(self)));
                    }else if v[0..4] == ['c','o','p','y'] {
                        return Ok(Object::Interface(copy(self)));
                    }else if v[0..4] == ['d','i','a','g'] {
                        return diag_slice(env,self);
                    }else if v[0..4] == ['l','i','s','t'] {
                        return Ok(List::new_object(list(self)));
                    }
                },
                5 => {
                    if v[0..5] == ['s','h','a','p','e'] {
                        return shape(self);
                    }
                },
                _ => {}
            }
            let t = &env.rte().interface_types.borrow()[interface_index::POLY_ARRAY];
            match t.get(key) {
                Some(value) => return Ok(value),
                None => {
                    env.index_error(&format!(
                        "Index error in Array.{0}: {0} not found.", key
                    ))
                }
            }
        }else{
            env.type_error("Type error in Array.x: x is not a string.")
        }
    }
    fn index(&self, indices: &[Object], env: &mut Env) -> FnResult {
        if self.n==1 && indices.len()==1 {
            match indices[0] {
                Object::Int(i) => {
                    let i = i as isize;
                    let a = self.data.borrow();
                    if i>=0 && (i as usize)<self.s[0].shape {
                        return Ok(a[(self.base as isize+self.s[0].stride*i) as usize].clone());
                    }else{
                        return env.index_error("Index error in a[i]: out of bounds.");
                    }
                },
                ref i => {
                    return env.type_error1("Type error in a[i]: i is not an integer.","i",i);
                }
            }
        }else if self.n==2 {
            if indices.len()==1 {
                let i = match indices[0] {
                    Object::Int(i) => i as isize,
                    ref i => return env.type_error1("Type error in a[i]: i is not an integer.","i",i)
                };
                if i>=0 && (i as usize)<self.s[0].shape {
                    let base = (self.base as isize+i*self.s[0].stride) as usize;
                    let stride = self.s[1].stride;
                    let shape = self.s[1].shape;
                    let a = Rc::new(Array{n: 1, base, data: self.data.clone(),
                        s: Box::new([ShapeStride{shape,stride}])});
                    return Ok(Object::Interface(a));
                }else{
                    return env.index_error("Index error in a[i]: i is out of bounds.");
                }
            }else if indices.len()==2 {
                let i = match indices[0] {
                    Object::Int(i) => i as isize,
                    ref i => return env.type_error1("Type error in a[i,j]: i is not an integer.","i",i)
                };
                let j = match indices[1] {
                    Object::Int(j) => j as isize,
                    ref j => return env.type_error1("Type error in a[i,j]: j is not an integer.","j",j)
                };
                if i>=0 && (i as usize)<self.s[0].shape &&
                   j>=0 && (j as usize)<self.s[1].shape
                {
                    let index = (self.base as isize+self.s[0].stride*i+self.s[1].stride*j) as usize;
                    return Ok(self.data.borrow()[index].clone());
                }else{
                    return env.index_error("Index error in a[i,j]: out of bounds.");
                }
            }else{
                return env.index_error("Index error in a[...]: shape does not fit.");     
            }
        }else{
            return env.index_error("Index error in a[...]: shape does not fit.");
        }
    }
    fn set_index(&self, indices: &[Object], value: &Object, env: &mut Env) -> FnResult {
        if self.n==1 && indices.len()==1 {
            match indices[0] {
                Object::Int(i) => {
                    let i = i as isize;
                    let mut data = self.data.borrow_mut();
                    if i>=0 && (i as usize)<self.s[0].shape {
                        let index = (self.base as isize+self.s[0].stride*i) as usize;
                        data[index] = value.clone();
                        return Ok(Object::Null);
                    }else{
                        return env.index_error(
                            "Index error in a[i]=x: out of bounds.");
                    }
                },
                ref i => {
                    return env.type_error1(
                        "Type error in a[i]=x: i is not an integer.","i",i);
                }
            }
        }else if self.n==2 {
            if indices.len()==1 {
                let i = match indices[0] {
                    Object::Int(i) => i as isize,
                    ref i => return env.type_error1(
                        "Type error in a[i]=v: i is not an integer.","i",i)
                };
                let v = if let Some(x) = downcast::<Array>(value) {x}
                else{
                    return env.type_error("Type error in a[i]=v: v is not an array.");
                };
                if v.n != 1 || v.s[0].shape != self.s[0].shape {
                    return env.type_error("Type error in a[i]=v: shapes of a[i] and v do not match.");
                }

                let (pdata,vbase,vstride) = if Rc::ptr_eq(&self.data,&v.data) {
                    let w = copy(&v);
                    (w.data.clone(),w.base as isize,w.s[0].stride)
                }else{
                    (v.data.clone(),v.base as isize,v.s[0].stride)
                };
                // if Rc::ptr_eq(&self.data,&v.data) {
                //     v = copy(&v);
                // }
                // let vbase = v.base as isize;
                // let vstride = v.s[0].stride;
                // let vdata = v.data.borrow();
                let vdata = pdata.borrow();
                
                if i>=0 && (i as usize)<self.s[0].shape {
                    let base = self.base as isize+i*self.s[0].stride;
                    let stride = self.s[1].stride;
                    let n = self.s[1].shape;
                    let mut data = self.data.borrow_mut();

                    for j in 0..n {
                        let index = (base+j as isize*stride) as usize;
                        let vindex = (vbase+j as isize*vstride) as usize;
                        data[index] = vdata[vindex].clone();
                    }
                    return Ok(Object::Null);
                }else{
                    return env.index_error("Index error in a[i]: i is out of bounds.");
                }
            }else if indices.len()==2 {
                let i = match indices[0] {
                    Object::Int(i) => i as isize,
                    ref i => return env.type_error1("Type error in a[i,j]: i is not an integer.","i",i)
                };
                let j = match indices[1] {
                    Object::Int(j) => j as isize,
                    ref j => return env.type_error1("Type error in a[i,j]: j is not an integer.","j",j)
                };
                if i>=0 && (i as usize)<self.s[0].shape &&
                   j>=0 && (j as usize)<self.s[1].shape
                {
                    let index = (self.base as isize+self.s[0].stride*i+self.s[1].stride*j) as usize;
                    let mut data = self.data.borrow_mut();
                    data[index] = value.clone();
                    return Ok(Object::Null);
                }else{
                    return env.index_error("Index error in a[i,j]: out of bounds.");
                }
            }else{
                return env.index_error("Index error in a[...]: shape does not fit.");     
            }
        }else{
            return env.index_error("Index error in a[...]: shape does not fit.");
        }
    }
    fn neg(self: Rc<Self>, env: &mut Env) -> FnResult {
        map_unary_operator(&self,&op_neg,'-',env)
    }
    fn add(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        map_binary_operator(&self,b,op_add,'+',env)
    }
    fn sub(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        map_binary_operator(&self,b,op_sub,'-',env)
    }
    fn rmul(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        scalar_multiplication(env,a,&self)
    }
    fn mul(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        if self.n==1 {
            if let Some(b) = downcast::<Array>(b) {
                if b.n==1 {
                    let m = self.s[0].shape.min(b.s[0].shape);
                    let adata = self.data.borrow();
                    let bdata = b.data.borrow();
                    return scalar_product(env,m,
                        &adata, self.base, self.s[0].stride,
                        &bdata, b.base, b.s[0].stride
                    );
                }
            }
            return scalar_multiplication(env,b,&self);
        }else if self.n==2 {
            if let Some(b) = downcast::<Array>(b) {
                if b.n==1 {
                    let m = self.s[1].shape.min(b.s[0].shape);
                    return mul_matrix_vector(env,m,&self,b);
                }else if b.n==2 {
                    if self.s[1].shape != b.s[0].shape {
                        return env.value_error(
                            "Value error in matrix multiplication A*B:\n  A.shape[0] != B.shape[1]."
                        );
                    }
                    let y = mul_matrix_matrix(env,self.s[1].shape,&self,b)?;
                    {
                        let data = y.data.borrow();
                        if data.len()==1 {
                            return Ok(data[0].clone());
                        }
                    }
                    return Ok(Object::Interface(y));
                }
            }
            return scalar_multiplication(env,b,&self);
        }else{
            return scalar_multiplication(env,b,&self);
        }
    }
    fn div(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        return scalar_division(env,&self,b);
    }
    fn pow(self: Rc<Self>, n: &Object, env: &mut Env) -> FnResult {
        if self.n==2 {
            if let Object::Int(n) = *n {
                let n = if n>=0 {n as u32} else {
                    panic!()
                };    
                if self.s[0].shape != self.s[1].shape {
                    panic!();
                }
                let y = matrix_power(env,&self,n,self.s[0].shape)?;
                return Ok(Object::Interface(y));
            }else{
                env.type_error1("Type error in A^n: n is not an integer.","n",&n)
            }
        }else{
            env.type_error("Type error in A^n: A is not a matrix.")
        }
    }
    fn eq(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        if let Some(b) = downcast::<Array>(b) {
            return compare(env,&self,b,"==",op_eq);
        }else{
            return Ok(Object::Bool(false));
        }
    }
    fn abs(self: Rc<Self>, env: &mut Env) -> FnResult {
        return abs(env,&self);
    }
}

fn compare(env: &mut Env, a: &Array, b: &Array,
    rs: &str,
    relation: fn(&mut Env, &Object, &Object)->FnResult
) -> FnResult
{
    if a.n == b.n {
        if a.n == 1 {
            if a.s[0].shape == b.s[0].shape {
                let adata = a.data.borrow();
                let bdata = b.data.borrow();
                let astride = a.s[0].stride;
                let bstride = b.s[0].stride;
                for i in 0..a.s[0].shape {
                    let aindex = (a.base as isize+i as isize*astride) as usize;
                    let bindex = (b.base as isize+i as isize*bstride) as usize;
                    let y = relation(env,&adata[aindex],&bdata[bindex])?;
                    match y {
                        Object::Bool(y) => {
                            if !y {return Ok(Object::Bool(false));}
                        },
                        _ => return env.type_error(
                            &format!("Type error in A{0}B: expected return value of '{0}' of type bool.",rs)
                        )
                    }
                }
                return Ok(Object::Bool(true));
            }
        }else if a.n == 2 {
            let m = a.s[0].shape;
            let n = a.s[1].shape;
            if m == b.s[0].shape && n == b.s[1].shape {
                let adata = a.data.borrow();
                let bdata = b.data.borrow();
                let aistride = a.s[0].stride;
                let ajstride = a.s[1].stride;
                let bistride = b.s[0].stride;
                let bjstride = b.s[1].stride;
                for i in 0..m {
                    let aibase = a.base as isize+i as isize*aistride;
                    let bibase = b.base as isize+i as isize*bistride;
                    for j in 0..n {
                        let aindex = (aibase+j as isize*ajstride) as usize;
                        let bindex = (bibase+j as isize*bjstride) as usize;
                        let y = relation(env,&adata[aindex],&bdata[bindex])?;
                        match y {
                            Object::Bool(y) => {
                                if !y {return Ok(Object::Bool(false));}
                            },
                            _ => return env.type_error(
                                &format!("Type error in A{0}B: expected return value of '{0}' of type bool.",rs)
                            )
                        }
                    }
                }
                return Ok(Object::Bool(true));      
            }
        }else{
            panic!();
        }
    }
    return Ok(Object::Bool(false));
}

fn copy_from_ref(a: &Array) -> Rc<Array> {
    Rc::new(Array{
        s: Box::new([
            ShapeStride{shape: a.s[0].shape, stride: a.s[0].stride},
            ShapeStride{shape: a.s[1].shape, stride: a.s[1].stride}
        ]),
        n: a.n, base: a.base,
        data: a.data.clone()
    })
}

fn scalar_matrix(n: usize, x: &Object, zero: &Object) -> Rc<Array> {
    let mut v: Vec<Object> = Vec::with_capacity(n*n);
    for i in 0..n {
        for j in 0..n {
            v.push(if i==j {x.clone()} else {zero.clone()});
        }
    }
    return Array::matrix(n,n,v);
}

fn matrix_power(env: &mut Env, a: &Array, mut n: u32, size: usize)
-> Result<Rc<Array>,Box<Exception>>
{
    let mut y = if n==0 {
        return Ok(scalar_matrix(size,&Object::Int(1),&Object::Int(0)));
    }else{
        n-=1;
        copy_from_ref(a)
    };
    let mut base = y.clone();
    loop {
        if n&1 == 1 {
            y = mul_matrix_matrix(env,size,&y,&base)?;
        }
        n /= 2;
        if n==0 {break;}
        base = mul_matrix_matrix(env,size,&base,&base)?;
    }
    return Ok(y);
}

fn map_unary_operator(a: &Array,
    operator: &dyn Fn(&mut Env,&Object) -> FnResult,
    _operator_symbol: char, env: &mut Env
) -> FnResult
{
    if a.n==1 {
        let mut v: Vec<Object> = Vec::with_capacity(a.s[0].shape);
        let stride = a.s[0].stride;
        let base = a.base as isize;
        let data = a.data.borrow();
        for i in 0..a.s[0].shape {
            let x = &data[(base+i as isize*stride) as usize];
            v.push(operator(env,x)?);
        }
        return Ok(Object::Interface(Array::vector(v)));
    }else if a.n==2 {
        let m = a.s[0].shape;
        let n = a.s[1].shape;
        let istride = a.s[0].stride;
        let jstride = a.s[1].stride;

        let mut v: Vec<Object> = Vec::with_capacity(m*n);
        let base = a.base as isize;
        let data = a.data.borrow();
        for i in 0..m {
            let jbase = base+i as isize*istride;
            for j in 0..n {
                let x = &data[(jbase+j as isize*jstride) as usize];
                v.push(operator(env,x)?);
            }
        }
        return Ok(Object::Interface(Array::matrix(m,n,v)));
    }else{
        panic!();
    }
}

fn map_binary_operator(a: &Array, b: &Object,
    operator: fn(&mut Env,&Object,&Object)->FnResult,
    operator_symbol: char, env: &mut Env
) -> FnResult
{
    if a.n==1 {
        let stride = a.s[0].stride;
        let base = a.base;
        let mut v: Vec<Object> = Vec::with_capacity(a.s[0].shape);
        if let Some(b) = downcast::<Array>(b) {
            if b.n != 1 {
                return env.type_error(&format!(
                    "Type error in v{}w: v is a vector, but w is of order {}.",
                    operator_symbol, b.n
                ));
            }
            if a.s[0].shape != b.s[0].shape {
                return env.type_error(&format!(
                    "Type error in v{}w: v is not of the same size as w.",
                    operator_symbol
                ));
            }
            let stride2 = b.s[0].stride;
            let base2 = b.base;
            let adata = a.data.borrow();
            let bdata = b.data.borrow();
            for i in 0..a.s[0].shape {
                let y = operator(env,
                    &adata[(base as isize+i as isize*stride) as usize],
                    &bdata[(base2 as isize+i as isize*stride2) as usize]
                )?;
                v.push(y);
            }
        }else{
            let adata = a.data.borrow();
            for i in 0..a.s[0].shape {
                let index = (base as isize+i as isize*stride) as usize;
                let y = operator(env,&adata[index],b)?;
                v.push(y);
            }
        }
        return Ok(Object::Interface(Array::vector(v)));
    }else if a.n==2 {
        if let Some(b) = downcast::<Array>(b) {
            if b.n != 2 {
                return env.type_error(&format!(
                    "Type error in A{}B: A is a matrix, but B is of order {}.",
                    operator_symbol, b.n
                ));
            }
            let m = a.s[0].shape;
            let n = a.s[1].shape;
            if m != b.s[0].shape || n != b.s[1].shape {
                return env.type_error(&format!(
                    "Type error in A{}B: A is not of the same shape as B.",
                    operator_symbol
                ));
            }
            let aistride = a.s[0].stride;
            let ajstride = a.s[1].stride;
            let bistride = b.s[0].stride;
            let bjstride = b.s[1].stride;
            let adata = a.data.borrow();
            let bdata = b.data.borrow();
            let mut v: Vec<Object> = Vec::with_capacity(m*n);
            for i in 0..m {
                let aibase = a.base as isize+i as isize*aistride;
                let bibase = b.base as isize+i as isize*bistride;
                for j in 0..n {
                    let aindex = (aibase+j as isize*ajstride) as usize;
                    let bindex = (bibase+j as isize*bjstride) as usize;
                    let y = operator(env,&adata[aindex],&bdata[bindex])?;
                    v.push(y);
                }
            }
            return Ok(Object::Interface(Array::matrix(m,n,v)));
        }else{
            panic!();
        }
    }else{
        panic!();
    }
}

fn map_plain(a: &Array, f: fn(&Object)->Object) -> Rc<Array> {
    if a.n==1 {
        let mut v: Vec<Object> = Vec::with_capacity(a.s[0].shape);
        let stride = a.s[0].stride;
        let base = a.base as isize;
        let data = a.data.borrow();
        for i in 0..a.s[0].shape {
            let x = &data[(base+i as isize*stride) as usize];
            v.push(f(x));
        }
        return Array::vector(v);
    }else if a.n==2 {
        let m = a.s[0].shape;
        let n = a.s[1].shape;
        let istride = a.s[0].stride;
        let jstride = a.s[1].stride;
        let base = a.base as isize;
        let data = a.data.borrow();
        let mut v: Vec<Object> = Vec::with_capacity(m*n);
        for i in 0..m {
            let jbase = base+i as isize*istride;
            for j in 0..n {
                let x = &data[(jbase+j as isize*jstride) as usize];
                v.push(f(x));
            }
        }
        return Array::matrix(m,n,v);
    }else{
        panic!();
    }
}

fn array_map(env: &mut Env, a: &Array, f: &Object) -> FnResult {
    if a.n==1 {
        let mut v: Vec<Object> = Vec::with_capacity(a.s[0].shape);
        let stride = a.s[0].stride;
        let base = a.base as isize;
        let data = a.data.borrow();
        for i in 0..a.s[0].shape {
            let x = data[(base+i as isize*stride) as usize].clone();
            let y = env.call(f,&Object::Null,&[x])?;
            v.push(y);
        }
        return Ok(Object::Interface(Array::vector(v)));
    }else if a.n==2 {
        let m = a.s[0].shape;
        let n = a.s[1].shape;
        let istride = a.s[0].stride;
        let jstride = a.s[1].stride;
        let base = a.base as isize;
        let data = a.data.borrow();
        let mut v: Vec<Object> = Vec::with_capacity(m*n);
        for i in 0..m {
            let jbase = base+i as isize*istride;
            for j in 0..n {
                let x = data[(jbase+j as isize*jstride) as usize].clone();
                let y = env.call(f,&Object::Null,&[x])?;
                v.push(y);
            }
        }
        return Ok(Object::Interface(Array::matrix(m,n,v)));
    }else{
        panic!();
    }
}

fn map(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"map")
    }
    if let Some(a) = downcast::<Array>(pself) {
        return array_map(env,a,&argv[0]);
    }else{
        panic!();
    }
}

fn copy(a: &Array) -> Rc<Array> {
    map_plain(a,|x| x.clone())
}

fn conj_element(x: &Object) -> Object {
    match *x {
        Object::Complex(z) => Object::Complex(Complex64{
            re: z.re, im: -z.im
        }),
        ref x => x.clone()
    }
}

fn abs_square_element(env: &mut Env, x: &Object) -> FnResult {
    match *x {
        Object::Complex(z) => Ok(Object::Float(z.re*z.re+z.im*z.im)),
        Object::Float(x) => Ok(Object::Float(x*x)),
        ref x => op_mul(env,x,x)
    }
}

fn conj(a: &Array) -> Rc<Array> {
    map_plain(a,conj_element)
}

fn transpose(a: &Array) -> Rc<Array> {
    if a.n==2 {
        let m = a.s[0].shape;
        let n = a.s[1].shape;
        Rc::new(Array{
            s: Box::new([
                ShapeStride{shape: n, stride: a.s[1].stride},
                ShapeStride{shape: m, stride: a.s[0].stride}
            ]),
            n: 2, base: 0,
            data: a.data.clone(),
        })
    }else{
        panic!();
    }
}

fn abs(env: &mut Env, a: &Array) -> FnResult {
    if a.n==1 {
        let base = a.base;
        let stride = a.s[0].stride;
        let data = a.data.borrow();
        let mut sum = abs_square_element(env,&data[base])?;
        for i in 1..a.s[0].shape {
            let index = (base as isize+i as isize*stride) as usize;
            let p = abs_square_element(env,&data[index])?;
            sum = op_add(env,&sum,&p)?;
        }
        return match sum {
            Object::Int(x) => Ok(Object::Float((x as f64).sqrt())),
            Object::Float(x) => Ok(Object::Float(x.sqrt())),
            _ => env.type_error("Type error in sqrt(v.abs).")
        };
    }else{
        panic!();
    }
}

fn list(a: &Array) -> Vec<Object> {
    if a.n==1 {
        let mut v: Vec<Object> = Vec::with_capacity(a.s[0].shape);
        let stride = a.s[0].stride;
        let base = a.base as isize;
        let data = a.data.borrow();
        for i in 0..a.s[0].shape {
            let x = &data[(base+i as isize*stride) as usize];
            v.push(x.clone());
        }
        return v;
    }else{
        panic!();
    }
}

fn scalar_multiplication(env: &mut Env,
    r: &Object, a: &Array
) -> FnResult
{
    let op = |env: &mut Env, x: &Object| -> FnResult {
        return op_mul(env,r,x);
    };
    return map_unary_operator(a,&op,'_',env);
}

fn scalar_division(env: &mut Env,
    a: &Array, r: &Object
) -> FnResult
{
    let op = |env: &mut Env, x: &Object| -> FnResult {
        return op_div(env,x,r);
    };
    return map_unary_operator(a,&op,'_',env);
}

fn scalar_product(env: &mut Env, n: usize,
    a: &[Object], abase: usize, astride: isize,
    b: &[Object], bbase: usize, bstride: isize
) -> FnResult
{
    let mut sum = op_mul(env,&a[abase],&b[bbase])?;
    for i in 1..n {
        let aindex = (abase as isize+i as isize*astride) as usize;
        let bindex = (bbase as isize+i as isize*bstride) as usize;
        let p = op_mul(env,&a[aindex],&b[bindex])?;
        sum = op_add(env,&sum,&p)?;
    }
    return Ok(sum);
}

fn mul_matrix_vector(env: &mut Env, n: usize,
    a: &Array, x: &Array
) -> FnResult
{
    let m = a.s[0].shape;
    let aistride = a.s[0].stride;
    let ajstride = a.s[1].stride;
    let xstride = x.s[0].stride;
    let abase = a.base;
    let xbase = x.base;
    let adata = a.data.borrow();
    let xdata = x.data.borrow();
    let mut y: Vec<Object> = Vec::with_capacity(m);
    for i in 0..m {
        let base = (abase as isize+i as isize*aistride) as usize;
        let p = scalar_product(env,n,
            &adata, base, ajstride,
            &xdata, xbase, xstride
        )?;
        y.push(p);
    }
    return Ok(Object::Interface(Array::vector(y)));
}

fn mul_matrix_matrix(env: &mut Env, size: usize,
    a: &Array, b: &Array
) -> Result<Rc<Array>,Box<Exception>>
{
    let m = a.s[0].shape;
    let n = b.s[1].shape;
    let aistride = a.s[0].stride;
    let akstride = a.s[1].stride;
    let bkstride = b.s[0].stride;
    let bjstride = b.s[1].stride;
    let adata = a.data.borrow();
    let bdata = b.data.borrow();
    let mut y: Vec<Object> = Vec::with_capacity(m*n);
    for i in 0..m {
        let ibase = (a.base as isize+i as isize*aistride) as usize;
        for j in 0..n {
            let jbase = (b.base as isize+j as isize*bjstride) as usize;
            let p = scalar_product(env,size,
                &adata, ibase, akstride,
                &bdata, jbase, bkstride
            )?;
            y.push(p);
        }
    }
    return Ok(Array::matrix(m,n,y));
}

fn shape(a: &Array) -> FnResult {
    let n = a.s.len();
    let mut v: Vec<Object> = Vec::with_capacity(n);
    for i in 0..n {
        v.push(Object::Int(a.s[i].shape as i32));
    }
    return Ok(List::new_object(v));
}

fn unit_vector(n: usize, k: usize) -> Rc<Array> {
    let mut v: Vec<Object> = Vec::with_capacity(n);
    for i in 0..n {
        v.push(Object::Int(if i==k {1} else {0}));
    }
    return Array::vector(v);
}

fn vector(_env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    return Ok(Object::Interface(Array::vector(Vec::from(argv))));
}

fn matrix_from_vectors(env: &mut Env, n: usize, argv: &[Object]) -> FnResult {
    let m = argv.len();
    let mut v: Vec<Object> = Vec::with_capacity(m*n);
    for t in argv {
        if let Some(a) = downcast::<Array>(t) {
            if a.n != 1 || a.s[0].shape != n {
                return env.value_error(
                "Vale error in matrix(*args): all args must be vectors of the same size.");
            }
            let base = a.base as isize;
            let stride = a.s[0].stride;
            let data = a.data.borrow();
            for i in 0..n {
                let index = (base+i as isize*stride) as usize;
                v.push(data[index].clone());
            }
        }else{
            return env.type_error(
            "Type error in matrix(*args): expected args of type Array.");            
        }
    }
    let y = Rc::new(Array{
        s: Box::new([
            ShapeStride{shape: m, stride: n as isize},
            ShapeStride{shape: n, stride: 1}
        ]),
        n: 2, base: 0,
        data: Rc::new(RefCell::new(v))
    });
    return Ok(Object::Interface(y));
}

fn matrix(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    'type_error: loop{
        let n = match argv[0] {
            Object::List(ref a) => a.borrow().v.len(),
            ref x => if let Some(x) = downcast::<Array>(x) {
                if x.n==1 {
                    return matrix_from_vectors(env,x.s[0].shape,argv);
                }else{
                    break 'type_error;
                }
            }else{
                break 'type_error
            }
        };
        let m = argv.len();
        let mut v: Vec<Object> = Vec::with_capacity(m*n);
        for t in argv {
            if let Object::List(ref list) = *t {
                let a = &list.borrow().v;
                if a.len() != n {
                    return env.value_error(
                    "Value error in matrix(*args): each args[k] must have the same size.");
                }
                for x in a {
                    v.push(x.clone());
                }
            }else{
                break 'type_error;
            }
        }
        return Ok(Object::Interface(Array::matrix(m,n,v)));
    }
    return env.type_error("Type error in matrix(*args): expected args of type List.");
}

fn array(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        2 => {}, argc => return env.argc_error(argc,2,2,"array")
    }
    let n = match argv[0] {
        Object::Int(x) => x as usize,
        _ => return env.type_error("Type error in array(n,a): n is not an integer.")
    };
    if n==1 {
        let y = crate::global::list(env,&argv[1])?;
        if let Object::List(a) = y {
            return Ok(Object::Interface(Array::vector(a.borrow().v.clone())));
        }else{
            panic!();
        }
    }else{
        return env.std_exception("Dimension not supported.");
    }
}

fn scalar(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        3 => {}, argc => return env.argc_error(argc,3,3,"scalar")
    }
    let n = match argv[0] {
        Object::Int(x) => if x<0 {0 as usize} else {x as usize},
        _ => return env.type_error1(
            "Type error in scalar(n,e,z): n is not an integer.","n",&argv[0])
    };
    let e = &argv[1];
    let z = &argv[2];
    return Ok(Object::Interface(scalar_matrix(n,e,z)));
}

fn unit(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        2 => {}, argc => return env.argc_error(argc,2,2,"unit")
    }
    let n = match argv[0] {
        Object::Int(x) => if x<0 {0 as usize} else {x as usize},
        _ => return env.type_error1(
            "Type error in unit(n,k): n is not an integer.","n",&argv[0])
    };
    let k = match argv[1] {
        Object::Int(x) => if x<0 {0 as usize} else {x as usize},
        _ => return env.type_error1(
            "Type error in unit(n,k): k is not an integer.","k",&argv[1])
    };
    return Ok(Object::Interface(unit_vector(n,k)));
}

fn diag(_env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    let n = argv.len();
    let mut v: Vec<Object> = Vec::with_capacity(n*n);
    let mut k = 0;
    for i in 0..n {
        for j in 0..n {
            if i==j {
                v.push(argv[k].clone());
                k+=1;
            }else{
                v.push(Object::Int(0));
            }
        }
    }
    return Ok(Object::Interface(Array::matrix(n,n,v)));
}

fn diag_slice(env: &mut Env, a: &Array) -> FnResult {
    let n = a.s[0].shape;
    if a.n==2 && n == a.s[1].shape {
        let mut v: Vec<Object> = Vec::with_capacity(n);
        let base = a.base as isize;
        let istride = a.s[0].stride;
        let jstride = a.s[1].stride;
        let data = a.data.borrow();
        for i in 0..n {
            let index = (base+i as isize*istride+i as isize*jstride) as usize;
            v.push(data[index].clone());
        }
        return Ok(List::new_object(v));
    }else{
        return env.type_error("Type error in A.diag: A is not a square matrix.");
    }
}

fn trace(env: &mut Env, a: &Array) -> FnResult {
    let n = a.s[0].shape;
    if a.n==2 && n == a.s[1].shape {
        let base = a.base as isize;
        let istride = a.s[0].stride;
        let jstride = a.s[1].stride;
        let data = a.data.borrow();
        let mut y = data[a.base].clone();
        for i in 1..n {
            let index = (base+i as isize*istride+i as isize*jstride) as usize;
            y = op_add(env,&y,&data[index])?;
        }
        return Ok(y);
    }else{
        return env.type_error("Type error in A.diag: A is not a square matrix.");
    }
}

fn la_copy(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, argc => return env.argc_error(argc,1,1,"id")
    }
    if let Some(a) = downcast::<Array>(&argv[0]) {
        return Ok(Object::Interface(copy(a)));
    }else{
        return Ok(argv[0].clone());
    }
}

pub fn load_math_la(env: &mut Env) -> Object
{
    let type_array = Table::new(Object::Null);
    {
        let mut m = type_array.map.borrow_mut();
        m.insert_fn_plain("map",map,1,1);
    }
    interface_types_set(env.rte(),interface_index::POLY_ARRAY,type_array);

    let la = new_module("la");
    {
        let mut m = la.map.borrow_mut();
        m.insert_fn_plain("vector",vector,0,VARIADIC);
        m.insert_fn_plain("matrix",matrix,0,VARIADIC);
        m.insert_fn_plain("array",array,2,2);
        m.insert_fn_plain("scalar",scalar,3,3);
        m.insert_fn_plain("unit",unit,2,2);
        m.insert_fn_plain("diag",diag,0,VARIADIC);
        m.insert_fn_plain("id",la_copy,1,1);
    }

    return Object::Interface(Rc::new(la));
}
