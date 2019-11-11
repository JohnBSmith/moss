

use std::ffi::{CString};
use std::os::raw::{c_int};
use std::mem;
use crate::sdl::{
    SDL_WINDOWPOS_CENTERED, SDL_WINDOW_SHOWN, SDL_RENDERER_ACCELERATED,
    SDL_KEYDOWN, SDL_KEYUP,
    Uint32, SDL_Scancode, SDL_Window, SDL_Renderer, SDL_Rect,
    SDL_Event, SDL_PollEvent, SDL_BlendMode,
    SDL_Delay, SDL_CreateWindow, SDL_CreateRenderer,
    SDL_SetRenderDrawColor, SDL_RenderClear, SDL_RenderPresent,
    SDL_RenderDrawPoint, SDL_RenderFillRect,
    SDL_Quit, SDL_DestroyWindow, SDL_SetRenderDrawBlendMode
};
use std::f64::consts::PI;

use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;

use crate::object::{
    Object, Interface, Function, FnResult, Map, Table, CharString,
    new_module, downcast
};
use crate::vm::Env;
use crate::data::Bytes;

#[path = "graphics/font.rs"]
mod font;

fn sleep(t: u32) {
    unsafe{SDL_Delay(t as Uint32);}
}

fn fade(x: f64) -> i32 {
    return (255.0*(-0.4*x*x*x).exp()) as i32;
}

fn fade_needle(x: f64) -> i32 {
    return (255.0*(-2.0*x*x*x).exp()) as i32;
}

#[inline(always)]
fn mod_floor(x: f64, m: f64) -> f64 {
    return x-m*(x/m).floor();
}

#[allow(non_snake_case)]
fn hsl_to_rgb(H: f64, S: f64, L: f64) -> (u8,u8,u8) {
    const TAU: f64 = 2.0*PI;
    let C = (1.0-(2.0*L-1.0).abs())*S;
    let Hp = 3.0*(mod_floor(H,TAU))/PI;
    let X = C*(1.0-(Hp%2.0-1.0).abs());
    let (R1,G1,B1)
    =    if Hp<1.0 {(C,X,0.0)}
    else if Hp<2.0 {(X,C,0.0)}
    else if Hp<3.0 {(0.0,C,X)}
    else if Hp<4.0 {(0.0,X,C)}
    else if Hp<5.0 {(X,0.0,C)}
    else if Hp<6.1 {(C,0.0,X)}
    else{(0.0,0.0,0.0)};
    let m = L-C/2.0;
    let R = 255.0*(R1+m);
    let G = 255.0*(G1+m);
    let B = 255.0*(B1+m);
    return (R as u8,G as u8,B as u8);
}

#[derive(Clone,Copy)]
struct Color {
    r: u8, g: u8, b: u8, a: u8
}

struct MutableCanvas {
    window: *mut SDL_Window,
    rdr: *mut SDL_Renderer,
    buffer: Box<[Color]>,
    width: u32, height: u32,
    px: i32, py: i32, wx: f64, wy: f64,
    color: Color
}

impl MutableCanvas {
    fn new(id: &str, w: usize, h: usize) -> MutableCanvas {
        let window = unsafe{
            SDL_CreateWindow(CString::new(id).unwrap().as_ptr(),
                SDL_WINDOWPOS_CENTERED as c_int, SDL_WINDOWPOS_CENTERED as c_int, 
                w as c_int, h as c_int, SDL_WINDOW_SHOWN)
        };
        let rdr = unsafe{
            SDL_CreateRenderer(window,0,SDL_RENDERER_ACCELERATED)
        };
        unsafe{
            SDL_SetRenderDrawBlendMode(rdr,SDL_BlendMode::BLEND);
            SDL_SetRenderDrawColor(rdr,0,0,0,255);
            SDL_RenderClear(rdr);
            SDL_SetRenderDrawColor(rdr,0xa0,0xa0,0xa0,255);
            SDL_RenderPresent(rdr);
        }
        let black = Color{r: 0, b: 0, g: 0, a: 0};
        let buffer = vec![black; w*h].into_boxed_slice();
        return MutableCanvas{
            window, rdr, buffer, width: w as u32, height: h as u32,
            px: w as i32/2, py: h as i32/2,
            wx: 0.5*w as f64, wy: 0.5*w as f64,
            color: Color{r: 0xa0, g: 0xa0, b: 0xa0, a: 255}
        };
    }

    fn flush(&mut self) {
        unsafe{SDL_RenderPresent(self.rdr);}
    }

    fn flush_vg_buffer(&mut self) {
        let mut index=0;
        for y in 0..self.height {
            for x in 0..self.width {
                let c = self.buffer[index];
                if c.a != 0 {
                    unsafe{
                        SDL_SetRenderDrawColor(self.rdr,c.r,c.g,c.b,c.a);
                        SDL_RenderDrawPoint(self.rdr,x as c_int,y as c_int);
                    }
                }
                index+=1;
            }
        }
    }

    fn flush_clear_vg_buffer(&mut self) {
        let mut index=0;
        for y in 0..self.height {
            for x in 0..self.width {
                let p = &mut self.buffer[index];
                let c = *p;
                *p = Color{r: 0, g: 0, b: 0, a: 0};
                if c.a != 0 {
                    unsafe{
                        SDL_SetRenderDrawColor(self.rdr,c.r,c.g,c.b,c.a);
                        SDL_RenderDrawPoint(self.rdr,x as c_int,y as c_int);
                    }
                }
                index+=1;
            }
        }
    }
    
    fn draw_graymap(&mut self, x: usize, y: usize, w: usize, h: usize, data: &[u8]) {
        let width = self.width as usize;
        let height = self.height as usize;
        for py in 0..h {
            let py_w = py*w;
            for px in 0..w {
                let byte = data[py_w+px];
                if byte<255 && x+px<width && y+py<height {
                    let c = &self.color;
                    unsafe{
                        SDL_SetRenderDrawColor(self.rdr,c.r,c.g,c.b,255-byte);
                        SDL_RenderDrawPoint(self.rdr,(x+px) as c_int,(y+py) as c_int);
                    }
                }
            }
        }
        let c = &self.color;
        unsafe{SDL_SetRenderDrawColor(self.rdr,c.r,c.g,c.b,c.a);}
    }

    fn draw_pixmap(&mut self, x: usize, y: usize, w: usize, h: usize, data: &[u8]) {
        let width = self.width as usize;
        let height = self.height as usize;
        let alpha = self.color.a;
        let w3 = w*3;
        for py in 0..h {
            let py_w3 = py*w3;
            for px in 0..w {
                if x+px<width && y+py<height {
                    let index = py_w3+px*3;
                    let r = data[index];
                    let g = data[index+1];
                    let b = data[index+2];
                    unsafe{
                        SDL_SetRenderDrawColor(self.rdr,r,g,b,alpha);
                        SDL_RenderDrawPoint(self.rdr,(x+px) as c_int,(y+py) as c_int);
                    }
                }
            }
        }
        let c = &self.color;
        unsafe{SDL_SetRenderDrawColor(self.rdr,c.r,c.g,c.b,c.a);}
    }

    fn pset(&mut self, x: u32, y: u32) {
        let w = self.width;
        let h = self.height;
        if x<w && y<h {
            let p = &mut self.buffer[(y*w+x) as usize];
            p.r = self.color.r;
            p.g = self.color.g;
            p.b = self.color.b;
            p.a = self.color.a;
        }
    }

    fn pseta(&mut self, x: u32, y: u32, a: i32) {
        let w = self.width;
        let h = self.height;
        if x<w && y<h {
            let p = &mut self.buffer[(y*w+x) as usize];
            if p.a == 0 {
                p.r = self.color.r;
                p.g = self.color.g;
                p.b = self.color.b;
                p.a = ((self.color.a as u32)*(a as u32)/255) as u8;
            }else{
                p.r = (p.r as i32+(self.color.r as i32-p.r as i32)*a/255) as u8;
                p.g = (p.g as i32+(self.color.g as i32-p.g as i32)*a/255) as u8;
                p.b = (p.b as i32+(self.color.b as i32-p.b as i32)*a/255) as u8;
                p.a = p.a.max(((self.color.a as u32)*(a as u32)/255) as u8);
            }
        }
    }

    fn rect(&mut self, x: u32, y: u32, w: u32, h: u32) {
        let r = SDL_Rect{
            x: x as c_int, y: y as c_int,
            w: w as c_int, h: h as c_int
        };
        unsafe{
            SDL_RenderFillRect(self.rdr,&r as *const SDL_Rect);
        }
    }

    fn point(&mut self, x: f64, y: f64) {
        let rx = x*self.wx;
        let ry = y*self.wy;
        let ix = rx as i32;
        let iy = ry as i32;
        for xi in -2..3 {
            for yj in -2..3 {
                let px = ix.wrapping_add(xi);
                let py = iy.wrapping_add(yj);
                let d = (px as f64-rx).hypot(py as f64-ry);
                let a = fade(d);
                let px = self.px.wrapping_add(px) as u32;
                let py = self.py.wrapping_sub(py) as u32;
                self.pseta(px,py,a);
            }
        }
    }

    fn needle(&mut self, x: f64, y: f64) {
        let rx = x*self.wx;
        let ry = y*self.wy;
        let ix = rx as i32;
        let iy = ry as i32;
        for xi in -1..2 {
            for yj in -1..2 {
                let px = ix.wrapping_add(xi);
                let py = iy.wrapping_add(yj);
                let d = (px as f64-rx).hypot(py as f64-ry);
                let a = fade_needle(d);
                let px = self.px.wrapping_add(px) as u32;
                let py = self.py.wrapping_sub(py) as u32;
                self.pseta(px,py,a);
            }
        }
    }

    fn circle(&mut self, x: f64, y: f64, radius: f64) {
        let step = 0.002/radius;
        let mut t=0.0;
        while t<2.0*PI {
            let vx = x+radius*t.cos();
            let vy = y+radius*t.sin();
            self.point(vx,vy);
            t+=step;
        }
    }

    fn disc(&mut self, x: f64, y: f64, radius: f64) {
        let radius_wx = radius*self.wx;
        let r = radius_wx.round() as i32;
        let ix = (x*self.wx).round() as i32;
        let iy = (y*self.wy).round() as i32;
        let radius_wx = radius_wx+0.2;
        for xi in -r..r+1 {
            for yj in -r..r+1 {
                if (xi as f64).hypot(yj as f64) < radius_wx {
                    let px = self.px.wrapping_add(xi).wrapping_add(ix) as u32;
                    let py = self.py.wrapping_sub(yj).wrapping_sub(iy) as u32;
                    self.pset(px,py);
                }
            }
        }
        self.circle(x,y,radius);
    }

    fn square(&mut self, x: f64, y: f64, inradius: f64) {
        let r = (inradius*self.wx).round() as i32+1;
        let ix = (x*self.wx).round() as i32;
        let iy = (y*self.wy).round() as i32;
        let px = self.px.wrapping_add(ix);
        let py = self.py.wrapping_sub(iy);
        for xi in -r..r+1 {
            let px = px.wrapping_add(xi) as u32;
            let py = py.wrapping_add(-r) as u32;
            self.pset(px,py);
            self.pset(px,py.wrapping_add(1));
        }
        for xi in -r..r+1 {
            let px = px.wrapping_add(xi) as u32;
            let py = py.wrapping_add(r) as u32;
            self.pset(px,py);
            self.pset(px,py.wrapping_sub(1));
        }
        for yi in -r..r+1 {
            let px = px.wrapping_add(-r) as u32;
            let py = py.wrapping_sub(yi) as u32;
            self.pset(px,py);
            self.pset(px.wrapping_add(1),py);
        }
        for yi in -r..r+1 {
            let px = px.wrapping_add(r) as u32;
            let py = py.wrapping_sub(yi) as u32;
            self.pset(px,py);
            self.pset(px.wrapping_sub(1),py);
        }
    }

    fn rgb(&mut self, r: f64, g: f64, b: f64, a: Option<f64>) {
        let ri = ((255.0*r) as i32).max(0).min(255) as u8;
        let gi = ((255.0*g) as i32).max(0).min(255) as u8;
        let bi = ((255.0*b) as i32).max(0).min(255) as u8;
        let ai = if let Some(a) = a {
            ((255.0*a) as i32).max(0).min(255) as u8
        }else{255};
        unsafe{SDL_SetRenderDrawColor(self.rdr,ri,gi,bi,ai);}
        let c = &mut self.color;
        c.r = ri; c.g = gi; c.b = bi; c.a = ai;
    }

    fn hsl(&mut self, h: f64, s: f64, l: f64, a: Option<f64>) {
        let (ri,gi,bi) = hsl_to_rgb(h,s,l);
        let ai = if let Some(a) = a {
            ((255.0*a) as i32).max(0).min(255) as u8
        }else{255};
        unsafe{SDL_SetRenderDrawColor(self.rdr,ri,gi,bi,ai);}
        let c = &mut self.color;
        c.r = ri; c.g = gi; c.b = bi; c.a = ai;
    }

    fn clear(&mut self, r: f64, g: f64, b: f64) {
        let ri = ((255.0*r) as i32).max(0).min(255) as u8;
        let gi = ((255.0*g) as i32).max(0).min(255) as u8;
        let bi = ((255.0*b) as i32).max(0).min(255) as u8;
        let c = self.color;
        unsafe{
            SDL_SetRenderDrawColor(self.rdr,ri,gi,bi,255);
            SDL_RenderClear(self.rdr);
            SDL_SetRenderDrawColor(self.rdr,c.r,c.g,c.b,c.a);
        }
    }
}

fn scancode_to_key(x: SDL_Scancode) -> &'static str {
    match x {
        SDL_Scancode::A => "a",
        SDL_Scancode::B => "b",
        SDL_Scancode::C => "c",
        SDL_Scancode::D => "d",
        SDL_Scancode::E => "e",
        SDL_Scancode::F => "f",
        SDL_Scancode::G => "g",
        SDL_Scancode::H => "h",
        SDL_Scancode::I => "i",
        SDL_Scancode::J => "j",
        SDL_Scancode::K => "k",
        SDL_Scancode::L => "l",
        SDL_Scancode::M => "m",
        SDL_Scancode::N => "n",
        SDL_Scancode::O => "o",
        SDL_Scancode::P => "p",
        SDL_Scancode::Q => "q",
        SDL_Scancode::R => "r",
        SDL_Scancode::S => "s",
        SDL_Scancode::T => "t",
        SDL_Scancode::U => "u",
        SDL_Scancode::V => "v",
        SDL_Scancode::W => "w",
        SDL_Scancode::X => "x",
        SDL_Scancode::Y => "y",
        SDL_Scancode::Z => "z",
        SDL_Scancode::N0 => "0",
        SDL_Scancode::N1 => "1",
        SDL_Scancode::N2 => "2",
        SDL_Scancode::N3 => "3",
        SDL_Scancode::N4 => "4",
        SDL_Scancode::N5 => "5",
        SDL_Scancode::N6 => "6",
        SDL_Scancode::N7 => "7",
        SDL_Scancode::N8 => "8",
        SDL_Scancode::N9 => "9",
        SDL_Scancode::RETURN => "return",
        SDL_Scancode::ESCAPE => "escape",
        SDL_Scancode::BACKSPACE => "backspace",
        SDL_Scancode::SPACE => "space",
        SDL_Scancode::TAB => "tab",
        SDL_Scancode::COMMA => ",",
        SDL_Scancode::MINUS => "-",
        SDL_Scancode::PERIOD => ".",
        SDL_Scancode::F1 => "F1",
        SDL_Scancode::F2 => "F2",
        SDL_Scancode::F3 => "F3",
        SDL_Scancode::F4 => "F4",
        SDL_Scancode::F5 => "F5",
        SDL_Scancode::F6 => "F6",
        SDL_Scancode::F7 => "F7",
        SDL_Scancode::F8 => "F8",
        SDL_Scancode::F9 => "F9",
        SDL_Scancode::F10 => "F10",
        SDL_Scancode::F11 => "F11",
        SDL_Scancode::F12 => "F12",
        SDL_Scancode::LEFT => "left",
        SDL_Scancode::RIGHT => "right",
        SDL_Scancode::UP => "up",
        SDL_Scancode::DOWN => "down",
        _ => "unknown",
    }
}

fn get_key() -> Object {
    unsafe{
        let mut event: SDL_Event = mem::uninitialized();
        while SDL_PollEvent(&mut event)!=0 {
            if event.event_type == SDL_KEYDOWN {
                let key = scancode_to_key(event.key.keysym.scancode);
                return CharString::new_object_str(key);
            }
        }
        return Object::Null;
    }
}

fn get_scancode() -> Object {
    unsafe{
        let mut event: SDL_Event = mem::uninitialized();
        while SDL_PollEvent(&mut event)!=0 {
            if event.event_type == SDL_KEYDOWN {
                return Object::Int(event.key.keysym.scancode as i32);
            }else if event.event_type == SDL_KEYUP {
                return Object::Int(event.key.keysym.scancode as i32+1000);
            }
        }
        return Object::Null;
    }
}

struct Canvas {
    canvas: RefCell<MutableCanvas>,
    type_canvas: Rc<Table>
}

impl Drop for Canvas {
    fn drop(&mut self) {
        let canvas = self.canvas.borrow_mut();
        unsafe{
            SDL_DestroyWindow(canvas.window);
            SDL_Quit();
        }
    }
}

impl Interface for Canvas {
    fn as_any(&self) -> &dyn Any {self}
    fn type_name(&self, _env: &mut Env) -> String {
        "Canvas".to_string()
    }
    fn get(self: Rc<Self>, key: &Object, env: &mut Env) -> FnResult {
        match self.type_canvas.map.borrow_mut().m.get(key) {
            Some(value) => return Ok(value.clone()),
            None => {
                env.index_error(&format!(
                    "Index error in Canvas.{0}: {0} not found.", key
                ))
            }
        }
    }
}

fn canvas_bind_type(type_canvas: Rc<Table>) -> Object {
    let canvas = Box::new(move |env: &mut Env, _pself: &Object, argv: &[Object]| -> FnResult {
        match argv.len() {
            2 => {}, n => return env.argc_error(n,2,2,"canvas")
        }
        let w = match argv[0] {
            Object::Int(w) => if w<0 {0} else {w as usize},
            ref w => return env.type_error1(
                "Type error in canvas(w,h): w is not an integer.","w",w)
        };
        let h = match argv[1] {
            Object::Int(h) => if h<0 {0} else {h as usize},
            ref h => return env.type_error1(
                "Type error in canvas(w,h): h is not an integer.","h",h)
        };
        let c = Canvas{
            canvas: RefCell::new(MutableCanvas::new("",w,h)),
            type_canvas: type_canvas.clone()
        };
        Ok(Object::Interface(Rc::new(c)))
    });
    return Function::mutable(canvas,2,2);
}

fn canvas_key(_env: &mut Env, _pself: &Object, _argv: &[Object]) -> FnResult {
    Ok(get_key())
}

fn canvas_scancode(_env: &mut Env, _pself: &Object, _argv: &[Object]) -> FnResult {
    Ok(get_scancode())
}

#[inline(never)]
fn type_error_canvas(env: &mut Env, app: &str, var: &str) -> FnResult {
    env.type_error(&format!("Type error in {}: {} is not of type Canvas.",app,var))
}

#[inline(never)]
fn type_error_int_float(env: &mut Env, fapp: &str, id: &str, x: &Object)
-> FnResult
{
    env.type_error1(&format!(
        "Type error in {}: {} shall be of type Int or Float",
    fapp,id),id,x)
}

fn canvas_flush(env: &mut Env, pself: &Object, _argv: &[Object]) -> FnResult {
    if let Some(canvas) = downcast::<Canvas>(pself) {
        let mut canvas = canvas.canvas.borrow_mut();
        canvas.flush();
        Ok(Object::Null)
    }else{
        type_error_canvas(env,"c.flush()","c")
    }
}

fn canvas_vflush(env: &mut Env, pself: &Object, _argv: &[Object]) -> FnResult {
    if let Some(canvas) = downcast::<Canvas>(pself) {
        let mut canvas = canvas.canvas.borrow_mut();
        canvas.flush_vg_buffer();
        Ok(Object::Null)
    }else{
        type_error_canvas(env,"c.vflush()","c")
    }
}

fn canvas_vcflush(env: &mut Env, pself: &Object, _argv: &[Object]) -> FnResult {
    if let Some(canvas) = downcast::<Canvas>(pself) {
        let mut canvas = canvas.canvas.borrow_mut();
        canvas.flush_clear_vg_buffer();
        Ok(Object::Null)
    }else{
        type_error_canvas(env,"c.vcflush()","c")
    }
}

fn canvas_point(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        2 => {}, n => return env.argc_error(n,2,2,"point")
    }
    let x = match argv[0] {
        Object::Int(x) => x as f64,
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"c.point(x,y)","x",x)
    };
    let y = match argv[1] {
        Object::Int(y) => y as f64,
        Object::Float(y) => y,
        ref y => return type_error_int_float(env,"c.point(x,y)","y",y)
    };
    if let Some(canvas) = downcast::<Canvas>(pself) {
        let mut canvas = canvas.canvas.borrow_mut();
        canvas.point(x,y);
        Ok(Object::Null)
    }else{
        type_error_canvas(env,"c.point(x,y)","c")
    }
}

fn canvas_needle(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        2 => {}, n => return env.argc_error(n,2,2,"needle")
    }
    let x = match argv[0] {
        Object::Int(x) => x as f64,
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"c.needle(x,y)","x",x)
    };
    let y = match argv[1] {
        Object::Int(y) => y as f64,
        Object::Float(y) => y,
        ref y => return type_error_int_float(env,"c.needle(x,y)","y",y)
    };
    if let Some(canvas) = downcast::<Canvas>(pself) {
        let mut canvas = canvas.canvas.borrow_mut();
        canvas.needle(x,y);
        Ok(Object::Null)
    }else{
        type_error_canvas(env,"c.needle(x,y)","c")
    }
}

fn canvas_circle(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        3 => {}, n => return env.argc_error(n,3,3,"circle")
    }
    let x = match argv[0] {
        Object::Int(x) => x as f64,
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"c.circle(x,y,r)","x",x)
    };
    let y = match argv[1] {
        Object::Int(y) => y as f64,
        Object::Float(y) => y,
        ref y => return type_error_int_float(env,"c.circle(x,y,r)","y",y)
    };
    let r = match argv[2] {
        Object::Int(y) => y as f64,
        Object::Float(y) => y,
        ref y => return type_error_int_float(env,"c.circle(x,y,r)","r",y)
    };
    if let Some(canvas) = downcast::<Canvas>(pself) {
        let mut canvas = canvas.canvas.borrow_mut();
        canvas.circle(x,y,r);
        Ok(Object::Null)
    }else{
        type_error_canvas(env,"c.circle(x,y,r)","c")
    }
}

fn canvas_disc(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        3 => {}, n => return env.argc_error(n,3,3,"disc")
    }
    let x = match argv[0] {
        Object::Int(x) => x as f64,
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"c.disc(x,y,r)","x",x)
    };
    let y = match argv[1] {
        Object::Int(y) => y as f64,
        Object::Float(y) => y,
        ref y => return type_error_int_float(env,"c.disc(x,y,r)","y",y)
    };
    let r = match argv[2] {
        Object::Int(y) => y as f64,
        Object::Float(y) => y,
        ref y => return type_error_int_float(env,"c.disc(x,y,r)","r",y)
    };
    if let Some(canvas) = downcast::<Canvas>(pself) {
        let mut canvas = canvas.canvas.borrow_mut();
        canvas.disc(x,y,r);
        Ok(Object::Null)
    }else{
        type_error_canvas(env,"c.disc(x,y,r)","c")
    }
}

fn canvas_box(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        3 => {}, n => return env.argc_error(n,3,3,"box")
    }
    let x = match argv[0] {
        Object::Int(x) => x as f64,
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"c.box(x,y,r)","x",x)
    };
    let y = match argv[1] {
        Object::Int(y) => y as f64,
        Object::Float(y) => y,
        ref y => return type_error_int_float(env,"c.box(x,y,r)","y",y)
    };
    let r = match argv[2] {
        Object::Int(y) => y as f64,
        Object::Float(y) => y,
        ref y => return type_error_int_float(env,"c.box(x,y,r)","r",y)
    };
    if let Some(canvas) = downcast::<Canvas>(pself) {
        let mut canvas = canvas.canvas.borrow_mut();
        canvas.square(x,y,r);
        Ok(Object::Null)
    }else{
        type_error_canvas(env,"c.box(x,y,r)","c")
    }
}

fn canvas_rgb(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    let a = match argv.len() {
        3 => None,
        4 => match argv[3] {
            Object::Int(a) => Some(a as f64),
            Object::Float(a) => Some(a),
            ref a => return type_error_int_float(env,"c.rgb(r,g,b,a)","a",a)
        },
        n => return env.argc_error(n,3,4,"rgb")
    };
    let r = match argv[0] {
        Object::Int(r) => r as f64,
        Object::Float(r) => r,
        ref r => return type_error_int_float(env,"c.rgb(r,g,b)","r",r)
    };
    let g = match argv[1] {
        Object::Int(g) => g as f64,
        Object::Float(g) => g,
        ref g => return type_error_int_float(env,"c.rgb(r,g,b)","g",g)
    };
    let b = match argv[2] {
        Object::Int(b) => b as f64,
        Object::Float(b) => b,
        ref b => return type_error_int_float(env,"c.rgb(r,g,b)","b",b)
    };
    if let Some(canvas) = downcast::<Canvas>(pself) {
        let mut canvas = canvas.canvas.borrow_mut();
        canvas.rgb(r,g,b,a);
        Ok(Object::Null)
    }else{
        type_error_canvas(env,"c.rgb(r,g,b)","c")
    }
}

fn canvas_hsl(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    let a = match argv.len() {
        3 => None,
        4 => match argv[3] {
            Object::Int(a) => Some(a as f64),
            Object::Float(a) => Some(a),
            ref a => return type_error_int_float(env,"c.hsl(h,s,l,a)","a",a)
        },
        n => return env.argc_error(n,3,4,"hsl")
    };
    let h = match argv[0] {
        Object::Int(x) => x as f64,
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"c.hsl(h,s,l)","h",x)
    };
    let s = match argv[1] {
        Object::Int(x) => x as f64,
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"c.hsl(h,s,l)","s",x)
    };
    let l = match argv[2] {
        Object::Int(x) => x as f64,
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"c.hsl(h,s,l)","l",x)
    };
    if let Some(canvas) = downcast::<Canvas>(pself) {
        let mut canvas = canvas.canvas.borrow_mut();
        canvas.hsl(h,s,l,a);
        Ok(Object::Null)
    }else{
        type_error_canvas(env,"c.hsl(h,s,l)","c")
    }
}

fn canvas_clear(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        3 => {}, n => return env.argc_error(n,3,3,"clear")
    }
    let r = match argv[0] {
        Object::Int(r) => r as f64,
        Object::Float(r) => r,
        ref r => return type_error_int_float(env,"c.clear(r,g,b)","r",r)
    };
    let g = match argv[1] {
        Object::Int(g) => g as f64,
        Object::Float(g) => g,
        ref g => return type_error_int_float(env,"c.clear(r,g,b)","g",g)
    };
    let b = match argv[2] {
        Object::Int(b) => b as f64,
        Object::Float(b) => b,
        ref b => return type_error_int_float(env,"c.clear(r,g,b)","b",b)
    };
    if let Some(canvas) = downcast::<Canvas>(pself) {
        let mut canvas = canvas.canvas.borrow_mut();
        canvas.clear(r,g,b);
        Ok(Object::Null)
    }else{
        type_error_canvas(env,"c.clear(r,g,b)","c")
    }
}

fn canvas_fill(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        4 => {}, n => return env.argc_error(n,4,4,"fill")
    }
    let x = match argv[0] {
        Object::Int(x) => x as u32,
        ref x => return env.type_error1(
            "Type error in c.fill(x,y,w,h): x is not an integer.","x",x)
    };
    let y = match argv[1] {
        Object::Int(y) => y as u32,
        ref y => return env.type_error1(
            "Type error in c.fill(x,y,w,h): y is not an integer.","y",y)
    };
    let w = match argv[2] {
        Object::Int(w) => w as u32,
        ref w => return env.type_error1(
            "Type error in c.fill(x,y,w,h): w is not an integer.","w",w)
    };
    let h = match argv[3] {
        Object::Int(h) => h as u32,
        ref h => return env.type_error1(
            "Type error in c.fill(x,y,w,h): h is not an integer.","h",h)
    };
    if let Some(canvas) = downcast::<Canvas>(pself) {
        let mut canvas = canvas.canvas.borrow_mut();
        canvas.rect(x,y,w,h);
        Ok(Object::Null)
    }else{
        type_error_canvas(env,"c.clear(r,g,b)","c")
    }
}

fn graphics_sleep(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"sleep")
    }
    let t = match argv[0] {
        Object::Int(x) => x as f64,
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"sleep(x)","x",x)
    };
    let ms = (1000.0*t) as u32;
    sleep(ms);
    return Ok(Object::Null);
}

fn canvas_scale(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        2 => {}, n => return env.argc_error(n,2,2,"scale")
    }
    let x = match argv[0] {
        Object::Int(x) => x as f64,
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"canvas.scale(x,y)","x",x)
    };
    let y = match argv[1] {
        Object::Int(y) => y as f64,
        Object::Float(y) => y,
        ref y => return type_error_int_float(env,"cancas.scale(x,y)","y",y)
    };
    if let Some(canvas) = downcast::<Canvas>(pself) {
        let mut canvas = canvas.canvas.borrow_mut();
        canvas.wx = x*0.5*canvas.width as f64;
        canvas.wy = y*0.5*canvas.width as f64;
        Ok(Object::Null)
    }else{
        type_error_canvas(env,"canvas.scale(x,y)","canvas")
    }
}

fn canvas_set_glyph(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    let x = match argv[0] {
        Object::Int(value) => value as usize,
        _ => panic!()
    };
    let y = match argv[1] {
        Object::Int(value) => value as usize,
        _ => panic!()
    };
    let data_obj = match downcast::<Bytes>(&argv[2]) {
        Some(value) => value,
        None => panic!()
    };
    let index = match argv[3] {
        Object::Int(value) => value as usize,
        _ => panic!()
    };
    let w = match argv[4] {
        Object::Int(value) => value as usize,
        _ => panic!()
    };
    let h = match argv[5] {
        Object::Int(value) => value as usize,
        _ => panic!()
    };
    if let Some(canvas) = downcast::<Canvas>(pself) {
        let mut canvas = canvas.canvas.borrow_mut();
        let data = data_obj.data.borrow_mut();
        let pdata = &data[index*w*h..];
        canvas.draw_graymap(x,y,w,h,pdata);
        return Ok(Object::Null);
    }else{
        type_error_canvas(env,"canvas.set_glyph","canvas")
    }
}

fn canvas_set_pixmap(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    let x = match argv[0] {
        Object::Int(value) => value as usize,
        _ => panic!()
    };
    let y = match argv[1] {
        Object::Int(value) => value as usize,
        _ => panic!()
    };
    let data_obj = match downcast::<Bytes>(&argv[2]) {
        Some(value) => value,
        None => panic!()
    };
    let w = match argv[3] {
        Object::Int(value) => value as usize,
        _ => panic!()
    };
    let h = match argv[4] {
        Object::Int(value) => value as usize,
        _ => panic!()
    };
    if let Some(canvas) = downcast::<Canvas>(pself) {
        let mut canvas = canvas.canvas.borrow_mut();
        let data = data_obj.data.borrow_mut();
        canvas.draw_pixmap(x,y,w,h,&data);
        return Ok(Object::Null);
    }else{
        type_error_canvas(env,"canvas.pixmap","canvas")
    }
}

fn load_img(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    let data = match downcast::<Bytes>(&argv[0]) {
        Some(value) => value,
        None => return env.type_error(
            "Type error in load_img_data(data): expected data: Data.")
    };
    let fmt = match argv[1] {
        Object::Int(index) => {
            match index {0 => font::Fmt::PGM, _ => font::Fmt::PPM}
        },
        _ => panic!()
    };
    let pdata = data.data.borrow_mut();

    let gdata = font::pnm_as_single_image(&pdata,fmt);
    let map = Map::new();
    {
        let mut m = map.borrow_mut();
        m.insert("width",Object::Int(gdata.width as i32));
        m.insert("height",Object::Int(gdata.height as i32));
        m.insert("data",Bytes::object_from_vec(gdata.data));
    }
    return Ok(Object::Map(map));
}

fn load_font(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    let data = match downcast::<Bytes>(&argv[0]) {
        Some(value) => value,
        None => return env.type_error(
            "Type error in load_font(data): expected data: Data.")
    };
    let cols = match argv[1] {
        Object::Int(value) => value as usize,
        _ => panic!()
    };
    let rows = match argv[2] {
        Object::Int(value) => value as usize,
        _ => panic!()
    };
    let w = match argv[3] {
        Object::Int(value) => value as usize,
        _ => panic!()
    };
    let h = match argv[4] {
        Object::Int(value) => value as usize,
        _ => panic!()
    };
    let shiftw = match argv[5] {
        Object::Int(value) => value as usize,
        _ => panic!()
    };
    let shifth = match argv[6] {
        Object::Int(value) => value as usize,
        _ => panic!()
    };
    let pdata = data.data.borrow_mut();
    let gdata = font::pgm_as_glyph_data(&pdata,cols,rows,w,h,shiftw,shifth);
    return Ok(Bytes::object_from_vec(gdata.data));
}

pub fn load_graphics() -> Object
{
    let type_canvas = Table::new(Object::Null);
    {
        let mut m = type_canvas.map.borrow_mut();
        m.insert_fn_plain("key",canvas_key,0,0);
        m.insert_fn_plain("scan",canvas_scancode,0,0);
        m.insert_fn_plain("flush",canvas_flush,0,0);
        m.insert_fn_plain("vflush",canvas_vflush,0,0);
        m.insert_fn_plain("vcflush",canvas_vcflush,0,0);
        m.insert_fn_plain("point",canvas_point,2,2);
        m.insert_fn_plain("needle",canvas_needle,2,2);
        m.insert_fn_plain("circle",canvas_circle,3,3);
        m.insert_fn_plain("disc",canvas_disc,3,3);
        m.insert_fn_plain("box",canvas_box,3,3);
        m.insert_fn_plain("rgb",canvas_rgb,3,4);
        m.insert_fn_plain("hsl",canvas_hsl,3,4);
        m.insert_fn_plain("clear",canvas_clear,3,3);
        m.insert_fn_plain("fill",canvas_fill,4,4);
        m.insert_fn_plain("scale",canvas_scale,2,2);
        m.insert_fn_plain("glyph",canvas_set_glyph,6,6);
        m.insert_fn_plain("pixmap",canvas_set_pixmap,2,2);
    }

    let graphics = new_module("graphics");
    {
        let mut m = graphics.map.borrow_mut();
        m.insert("Canvas",Object::Interface(type_canvas.clone()));
        m.insert("canvas",canvas_bind_type(type_canvas));
        m.insert_fn_plain("sleep",graphics_sleep,1,1);
        m.insert_fn_plain("load_font",load_font,7,7);
        m.insert_fn_plain("load_img_data",load_img,2,2);
    }
    
    // Workaround for a bug in SDL on Ubuntu 18.04 on i386.
    // "arguments to dbus_message_new_method_call() were incorrect"
    // https://bugs.launchpad.net/ubuntu/+source/libsdl2/+bug/1775067
    ::std::env::set_var("DBUS_FATAL_WARNINGS","0");

    return Object::Interface(Rc::new(graphics));
}

