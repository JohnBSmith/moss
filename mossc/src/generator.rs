
#![allow(dead_code)]

use std::rc::Rc;
use std::collections::HashMap;
use std::mem::replace;
use parser::{AST, Symbol, Info};
use typing::{SymbolTable,VariableKind};

// byte code size
// byte code+argument size
// byte code+argument+argument size
pub const BCSIZE: usize = 1;
pub const BCASIZE: usize = 2;
pub const BCAASIZE: usize = 3;

const VARIADIC: u32 = 0xffffffff;

#[allow(dead_code)]
pub mod bc{
    pub const NULL: u8 = 00;
    pub const OF:   u8 = 01;
    pub const FALSE:u8 = 02;
    pub const TRUE: u8 = 03;
    pub const INT:  u8 = 04;
    pub const FLOAT:u8 = 05;
    pub const IMAG: u8 = 06;
    pub const NEG:  u8 = 07;
    pub const ADD:  u8 = 08;
    pub const SUB:  u8 = 09;
    pub const MUL:  u8 = 10;
    pub const DIV:  u8 = 11;
    pub const IDIV: u8 = 12;
    pub const POW:  u8 = 13;
    pub const EQ:   u8 = 14;
    pub const NE:   u8 = 15;
    pub const IS:   u8 = 16;
    pub const ISNOT:u8 = 17;
    pub const IN:   u8 = 18;
    pub const NOTIN:u8 = 19; 
    pub const LT:   u8 = 20;
    pub const GT:   u8 = 21;
    pub const LE:   u8 = 22;
    pub const GE:   u8 = 23;
    pub const LIST: u8 = 24;
    pub const MAP:  u8 = 25;
    pub const LOAD: u8 = 26;
    pub const STORE:u8 = 27;
    pub const JMP:  u8 = 28;
    pub const JZ:   u8 = 29;
    pub const JNZ:  u8 = 30;
    pub const CALL: u8 = 31;
    pub const RET:  u8 = 32;
    pub const STR:  u8 = 33;
    pub const FN:   u8 = 34;
    pub const FNSEP:u8 = 35;
    pub const POP:  u8 = 36;
    pub const LOAD_LOCAL: u8 = 37;
    pub const LOAD_ARG: u8 = 38;
    pub const LOAD_CONTEXT: u8 = 39;
    pub const STORE_LOCAL: u8 = 40;
    pub const STORE_ARG: u8 = 41;
    pub const STORE_CONTEXT: u8 = 42;
    pub const FNSELF: u8 = 43;
    pub const GET_INDEX: u8 = 44;
    pub const SET_INDEX: u8 = 45;
    pub const DOT:  u8 = 46;
    pub const DOT_SET: u8 = 47;
    pub const SWAP: u8 = 48;
    pub const DUP:  u8 = 49;
    pub const DUP_DOT_SWAP: u8 = 50;
    pub const AND:  u8 = 51;
    pub const OR:   u8 = 52;
    pub const NOT:  u8 = 53;
    pub const NEXT: u8 = 54;
    pub const RANGE:u8 = 55;
    pub const MOD:  u8 = 56;
    pub const ELSE: u8 = 57;
    pub const YIELD:u8 = 58;
    pub const EMPTY:u8 = 59;
    pub const TABLE:u8 = 60;
    pub const GET:  u8 = 61;
    pub const BAND: u8 = 62;
    pub const BOR:  u8 = 63;
    pub const BXOR: u8 = 64;
    pub const AOP:  u8 = 65;
    pub const RAISE:u8 = 66;
    pub const AOP_INDEX:u8 = 67;
    pub const OP:   u8 = 68;
    pub const TRY:  u8 = 69;
    pub const TRYEND:u8 = 70;
    pub const GETEXC:u8 = 71;
    pub const CRAISE:u8 = 72;
    pub const HALT: u8 = 73;
    pub const LONG: u8 = 74;
    pub const TUPLE:u8 = 75;
    pub const APPLY:u8 = 76;
}

fn push_bc(bv: &mut Vec<u32>, byte: u8, line: usize, col: usize) {
    bv.push(((col as u32)&0xff)<<24 | ((line as u32)&0xffff)<<8 | (byte as u32));
}

fn push_u32(bv: &mut Vec<u32>, x: u32){
    bv.push(x);
}

fn push_i32(bv: &mut Vec<u32>, x: i32){
    bv.push(x as u32);
}

fn load_i32(a: &[u32]) -> i32 {
    a[0] as i32
}

fn write_i32(a: &mut [u32], x: i32){
    a[0] = x as u32;
}

pub struct Pool{
    stab: HashMap<Rc<str>,usize>,
    stab_index: usize,
    data: Vec<Rc<str>>
}

impl Pool{
    fn get_index(&mut self, key: &str) -> usize {
        if let Some(index) = self.stab.get(key) {
            return *index;
        }
        let pkey: Rc<str> = Rc::from(key);
        self.data.push(pkey.clone());
        self.stab.insert(pkey,self.stab_index);
        self.stab_index+=1;
        return self.stab_index-1;
    }
}

struct Generator {
    pool: Pool,
    symbol_table: SymbolTable,
    bv_blocks: Vec<u32>,
    fn_indices: Vec<usize>
}

impl Generator {

fn offsets(&self, bv: &mut Vec<u32>, offset: i32){
    for &index in &self.fn_indices {
        let x = load_i32(&bv[index..index+1]);
        write_i32(&mut bv[index..index+1], x+BCSIZE as i32+offset-index as i32);
    }
}

fn compile_identifier(&mut self, bv: &mut Vec<u32>, t: &AST, id: &str) {
    if let Some(info) = self.symbol_table.get(id) {
        match info.kind {
            VariableKind::Global => {
                let index = self.pool.get_index(id);
                push_bc(bv,bc::LOAD,t.line,t.col);
                push_u32(bv,index as u32);
            },
            VariableKind::Argument(index) => {
                push_bc(bv,bc::LOAD_ARG,t.line,t.col);
                push_u32(bv,index as u32);
            },
            _ => panic!()
        }
    }else{
        unreachable!();
    }
}

fn compile_string(&mut self, bv: &mut Vec<u32>, t: &AST, id: &str) {
    let index = self.pool.get_index(id);
    push_bc(bv,bc::STR,t.line,t.col);
    push_u32(bv,index as u32);
}

fn compile_application(&mut self, bv: &mut Vec<u32>, t: &AST) {
    let a = t.argv();
    let self_argument = false;
    let n = a.len();

    let argc = if self_argument {n-2} else {n-1};

    if self_argument {
        // callee
        self.compile_node(bv,&a[0]);
    }else if a[0].value == Symbol::Dot {
        let b = a[0].argv();
        self.compile_node(bv,&b[0]);
        self.compile_node(bv,&b[1]);
        push_bc(bv, bc::DUP_DOT_SWAP, t.line, t.col);
    }else{
        // callee
        self.compile_node(bv,&a[0]);

        // self argument
        push_bc(bv, bc::NULL, t.line, t.col);
    }

    // arguments
    for i in 1..a.len() {
        self.compile_node(bv,&a[i]);
    }

    push_bc(bv, bc::CALL, t.line, t.col);

    // argument count,
    // not counting the self argument,
    // not counting the callee
    push_u32(bv, argc as u32);
}

fn compile_binary_operator(&mut self, bv: &mut Vec<u32>, t: &AST, code: u8) {
    let a = t.argv();
    self.compile_node(bv,&a[0]);
    self.compile_node(bv,&a[1]);
    push_bc(bv,code,t.line,t.col);
}

fn compile_let_statement(&mut self, bv: &mut Vec<u32>, t: &AST) {
    let a = t.argv();
    let key = match a[0].info {Info::Id(ref value)=>value, _ => panic!()};

    self.compile_node(bv,&a[2]);

    let index = self.pool.get_index(key);
    push_bc(bv,bc::STORE,t.line,t.col);
    push_u32(bv,index as u32);
}

fn compile_list_literal(&mut self, bv: &mut Vec<u32>, t: &AST) {
    let a = t.argv();
    for x in a {
        self.compile_node(bv,x);
    }
    push_bc(bv,bc::LIST,t.line,t.col);
    
    // #overflow
    push_u32(bv,a.len() as u32);
}

fn compile_fn(&mut self, bv: &mut Vec<u32>, t: &AST) {
    let header = match t.info {
        Info::FnHeader(ref value) => value, _ => unreachable!()
    };
    let body = &t.argv()[0];
    let mut bv2: Vec<u32> = Vec::new();

    let selfarg = false;
    let variadic = false;

    // A separator to identify a new code block. Just needed
    // to make the assembler listing human readable.
    push_bc(&mut bv2, bc::FNSEP, t.line, t.col);

    // Move self.fn_indices beside to allow nested functions.
    let fn_indices = replace(&mut self.fn_indices,Vec::new());
    // let jmp_stack = replace(&mut self.jmp_stack,Vec::new());

    // Every function has its own table of variables.
    let stab = header.symbol_table.borrow_mut().take().unwrap();
    let context = replace(&mut self.symbol_table,stab);
    self.symbol_table.context = Some(Box::new(context));

    let count_optional = 0;

    // Compile the function body.
    self.compile_node(&mut bv2,body);

    let var_count = self.symbol_table.count();

    // Shift the start adresses of nested functions
    // by the now known offset and turn them into
    // position independent code. The offset is negative
    // because the code blocks of nested functions come
    // before this code block. So we need to jump back.
    self.offsets(&mut bv2,-(self.bv_blocks.len() as i32));

    // Restore self.fn_indices.
    replace(&mut self.fn_indices,fn_indices);
    // self.jmp_stack = jmp_stack;

    // Add an additional return statement that will be reached
    // in case the control flow reaches the end of the function.
    push_bc(&mut bv2, bc::NULL, t.line, t.col);
    push_bc(&mut bv2, bc::RET, t.line, t.col);

    // Closure bindings, unimplemented.
    push_bc(bv, bc::NULL, t.line, t.col);

    // Restore.
    if let Some(context) = self.symbol_table.context.take() {
        self.symbol_table = *context;
    }

    // The name of the function.
    match header.id {
        Some(ref s) => {
            let index = self.pool.get_index(s);
            push_bc(bv,bc::STR,t.line,t.col);
            push_u32(bv,index as u32);
        },
        None => {
            push_bc(bv,bc::INT,t.line,t.col);
            push_u32(bv,((t.col as u32 & 0xffff)<<16) | (t.line as u32 & 0xffff));
        }
    }

    // Function constructor instruction.
    push_bc(bv, bc::FN, t.line, t.col);

    // Start address of the function body.
    // Add +1 to point behind FNSEP.
    // The size of bv will be added as the
    // compilation is finished.
    let index = bv.len();
    self.fn_indices.push(index);
    push_u32(bv,self.bv_blocks.len() as u32+1);

    let argc = if selfarg {
        header.argv.len()-1
    }else{
        header.argv.len()
    };

    if variadic {
        push_u32(bv,(argc-1) as u32);
        push_u32(bv,VARIADIC);
    }else{
        // minimal argument count
        push_u32(bv,(argc-count_optional) as u32);

        // maximal argument count
        push_u32(bv,argc as u32);
    }

    // number of local variables
    push_u32(bv,var_count as u32);

    // Append the code block to the buffer of code blocks.
    self.bv_blocks.append(&mut bv2);

}

fn compile_return(&mut self, bv: &mut Vec<u32>, t: &AST) {
    let a = t.argv();
    if a.len()==0 {
        push_bc(bv,bc::NULL,t.line,t.col);
    }else{
        self.compile_node(bv,&a[0]);
    }
    push_bc(bv,bc::RET,t.line,t.col);
}

fn compile_node(&mut self, bv: &mut Vec<u32>, t: &AST) {
    match t.value {
        Symbol::Item => {
            match t.info {
                Info::Int(value) => {
                    push_bc(bv,bc::INT,t.line,t.col);
                    push_i32(bv,value);
                },
                Info::Id(ref id) => {
                    self.compile_identifier(bv,t,id);
                },
                Info::String(ref s) => {
                    self.compile_string(bv,t,s);
                },
                _ => unimplemented!()
            }
        },
        Symbol::Block => {
            let a = t.argv();
            for x in a {
                self.compile_node(bv,x);
            }
        },
        Symbol::Function => {
            self.compile_fn(bv,t);
        },
        Symbol::Return => {
            self.compile_return(bv,t);
        },
        Symbol::Let => {
            self.compile_let_statement(bv,t);
        },
        Symbol::Application => {
            self.compile_application(bv,t);
        },
        Symbol::List => {
            self.compile_list_literal(bv,t);
        },
        Symbol::Plus => {
            self.compile_binary_operator(bv,t,bc::ADD);
        },
        Symbol::Minus => {
            self.compile_binary_operator(bv,t,bc::SUB);
        },
        Symbol::Ast => {
            self.compile_binary_operator(bv,t,bc::MUL);
        },
        Symbol::Pow => {
            self.compile_binary_operator(bv,t,bc::POW);
        },
        _ => unimplemented!("{}",t.value)
    }
}

}

pub struct CodeObject {
    pub program: Vec<u32>,
    pub data: Vec<Rc<str>> 
}

pub fn generate(t: &AST, stab: SymbolTable) -> CodeObject {
    let mut bv: Vec<u32> = Vec::new();
    let pool = Pool{stab: HashMap::new(), stab_index: 0, data: Vec::new()};
    let mut gen = Generator{
        pool,
        symbol_table: stab,
        bv_blocks: Vec::new(),
        fn_indices: Vec::new()
    };
    gen.compile_node(&mut bv,t);
    push_bc(&mut bv,bc::HALT,0,0);
    let len = bv.len();
    gen.offsets(&mut bv, len as i32);
    bv.append(&mut gen.bv_blocks);

    return CodeObject{program: bv, data: gen.pool.data};
}

