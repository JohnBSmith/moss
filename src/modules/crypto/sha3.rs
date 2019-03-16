
/*
SHA3-256

Original implemntation in C:
https://github.com/coruus/keccak-tiny
by: David Leon Gil

Port to rust:
Marek Kotewicz (marek.kotewicz@gmail.com)

License: CC0.
No liability.

Adapted by:
Finn (2019-03-16)
*/

const RHO: [u32; 24] = [
     1,  3,  6, 10, 15, 21,
    28, 36, 45, 55,  2, 14,
    27, 41, 56,  8, 25, 43,
    62, 18, 39, 61, 20, 44
];

const PI: [usize; 24] = [
    10,  7, 11, 17, 18, 3,
     5, 16,  8, 21, 24, 4,
    15, 23, 19, 13, 12, 2,
    20, 14, 22,  9,  6, 1
];

const RC: [u64; 24] = [
    1u64, 0x8082u64, 0x800000000000808au64, 0x8000000080008000u64,
    0x808bu64, 0x80000001u64, 0x8000000080008081u64, 0x8000000000008009u64,
    0x8au64, 0x88u64, 0x80008009u64, 0x8000000au64,
    0x8000808bu64, 0x800000000000008bu64, 0x8000000000008089u64, 0x8000000000008003u64,
    0x8000000000008002u64, 0x8000000000000080u64, 0x800au64, 0x800000008000000au64,
    0x8000000080008081u64, 0x8000000000008080u64, 0x80000001u64, 0x8000000080008008u64
];

// keccak-f[1600]
pub fn keccakf(a: &mut [u64; PLEN]) {
    for i in 0..24 {
        let mut array: [u64; 5] = [0; 5];

        // Theta
        array[0] ^= a[0 + 0*5];
        array[0] ^= a[0 + 1*5];
        array[0] ^= a[0 + 2*5];
        array[0] ^= a[0 + 3*5];
        array[0] ^= a[0 + 4*5];

        array[1] ^= a[1 + 0*5];
        array[1] ^= a[1 + 1*5];
        array[1] ^= a[1 + 2*5];
        array[1] ^= a[1 + 3*5];
        array[1] ^= a[1 + 4*5];

        array[2] ^= a[2 + 0*5];
        array[2] ^= a[2 + 1*5];
        array[2] ^= a[2 + 2*5];
        array[2] ^= a[2 + 3*5];
        array[2] ^= a[2 + 4*5];

        array[3] ^= a[3 + 0*5];
        array[3] ^= a[3 + 1*5];
        array[3] ^= a[3 + 2*5];
        array[3] ^= a[3 + 3*5];
        array[3] ^= a[3 + 4*5];

        array[4] ^= a[4 + 0*5];
        array[4] ^= a[4 + 1*5];
        array[4] ^= a[4 + 2*5];
        array[4] ^= a[4 + 3*5];
        array[4] ^= a[4 + 4*5];

        a[0*5 + 0] ^= array[(0+4)%5] ^ array[(0+1)%5].rotate_left(1);
        a[1*5 + 0] ^= array[(0+4)%5] ^ array[(0+1)%5].rotate_left(1);
        a[2*5 + 0] ^= array[(0+4)%5] ^ array[(0+1)%5].rotate_left(1);
        a[3*5 + 0] ^= array[(0+4)%5] ^ array[(0+1)%5].rotate_left(1);
        a[4*5 + 0] ^= array[(0+4)%5] ^ array[(0+1)%5].rotate_left(1);

        a[0*5 + 1] ^= array[(1+4)%5] ^ array[(1+1)%5].rotate_left(1);
        a[1*5 + 1] ^= array[(1+4)%5] ^ array[(1+1)%5].rotate_left(1);
        a[2*5 + 1] ^= array[(1+4)%5] ^ array[(1+1)%5].rotate_left(1);
        a[3*5 + 1] ^= array[(1+4)%5] ^ array[(1+1)%5].rotate_left(1);
        a[4*5 + 1] ^= array[(1+4)%5] ^ array[(1+1)%5].rotate_left(1);

        a[0*5 + 2] ^= array[(2+4)%5] ^ array[(2+1)%5].rotate_left(1);
        a[1*5 + 2] ^= array[(2+4)%5] ^ array[(2+1)%5].rotate_left(1);
        a[2*5 + 2] ^= array[(2+4)%5] ^ array[(2+1)%5].rotate_left(1);
        a[3*5 + 2] ^= array[(2+4)%5] ^ array[(2+1)%5].rotate_left(1);
        a[4*5 + 2] ^= array[(2+4)%5] ^ array[(2+1)%5].rotate_left(1);

        a[0*5 + 3] ^= array[(3+4)%5] ^ array[(3+1)%5].rotate_left(1);
        a[1*5 + 3] ^= array[(3+4)%5] ^ array[(3+1)%5].rotate_left(1);
        a[2*5 + 3] ^= array[(3+4)%5] ^ array[(3+1)%5].rotate_left(1);
        a[3*5 + 3] ^= array[(3+4)%5] ^ array[(3+1)%5].rotate_left(1);
        a[4*5 + 3] ^= array[(3+4)%5] ^ array[(3+1)%5].rotate_left(1);

        a[0*5 + 4] ^= array[(4+4)%5] ^ array[(4+1)%5].rotate_left(1);
        a[1*5 + 4] ^= array[(4+4)%5] ^ array[(4+1)%5].rotate_left(1);
        a[2*5 + 4] ^= array[(4+4)%5] ^ array[(4+1)%5].rotate_left(1);
        a[3*5 + 4] ^= array[(4+4)%5] ^ array[(4+1)%5].rotate_left(1);
        a[4*5 + 4] ^= array[(4+4)%5] ^ array[(4+1)%5].rotate_left(1);

        // Rho and pi
        let mut last = a[1];
        array[0] = a[PI[0]]; a[PI[0]] = last.rotate_left(RHO[0]); last = array[0];
        array[0] = a[PI[1]]; a[PI[1]] = last.rotate_left(RHO[1]); last = array[0];
        array[0] = a[PI[2]]; a[PI[2]] = last.rotate_left(RHO[2]); last = array[0];
        array[0] = a[PI[3]]; a[PI[3]] = last.rotate_left(RHO[3]); last = array[0];
        array[0] = a[PI[4]]; a[PI[4]] = last.rotate_left(RHO[4]); last = array[0];

        array[0] = a[PI[5]]; a[PI[5]] = last.rotate_left(RHO[5]); last = array[0];
        array[0] = a[PI[6]]; a[PI[6]] = last.rotate_left(RHO[6]); last = array[0];
        array[0] = a[PI[7]]; a[PI[7]] = last.rotate_left(RHO[7]); last = array[0];
        array[0] = a[PI[8]]; a[PI[8]] = last.rotate_left(RHO[8]); last = array[0];
        array[0] = a[PI[9]]; a[PI[9]] = last.rotate_left(RHO[9]); last = array[0];

        array[0] = a[PI[10]]; a[PI[10]] = last.rotate_left(RHO[10]); last = array[0];
        array[0] = a[PI[11]]; a[PI[11]] = last.rotate_left(RHO[11]); last = array[0];
        array[0] = a[PI[12]]; a[PI[12]] = last.rotate_left(RHO[12]); last = array[0];
        array[0] = a[PI[13]]; a[PI[13]] = last.rotate_left(RHO[13]); last = array[0];
        array[0] = a[PI[14]]; a[PI[14]] = last.rotate_left(RHO[14]); last = array[0];

        array[0] = a[PI[15]]; a[PI[15]] = last.rotate_left(RHO[15]); last = array[0];
        array[0] = a[PI[16]]; a[PI[16]] = last.rotate_left(RHO[16]); last = array[0];
        array[0] = a[PI[17]]; a[PI[17]] = last.rotate_left(RHO[17]); last = array[0];
        array[0] = a[PI[18]]; a[PI[18]] = last.rotate_left(RHO[18]); last = array[0];
        array[0] = a[PI[19]]; a[PI[19]] = last.rotate_left(RHO[19]); last = array[0];

        array[0] = a[PI[20]]; a[PI[20]] = last.rotate_left(RHO[20]); last = array[0];
        array[0] = a[PI[21]]; a[PI[21]] = last.rotate_left(RHO[21]); last = array[0];
        array[0] = a[PI[22]]; a[PI[22]] = last.rotate_left(RHO[22]); last = array[0];
        array[0] = a[PI[23]]; a[PI[23]] = last.rotate_left(RHO[23]);

        // Chi
        let y = 0*5;
        array[0] = a[y + 0];
        array[1] = a[y + 1];
        array[2] = a[y + 2];
        array[3] = a[y + 3];
        array[4] = a[y + 4];
        a[y + 0] = array[0] ^ ((!array[(0+1)%5]) & (array[(0+2)%5]));
        a[y + 1] = array[1] ^ ((!array[(1+1)%5]) & (array[(1+2)%5]));
        a[y + 2] = array[2] ^ ((!array[(2+1)%5]) & (array[(2+2)%5]));
        a[y + 3] = array[3] ^ ((!array[(3+1)%5]) & (array[(3+2)%5]));
        a[y + 4] = array[4] ^ ((!array[(4+1)%5]) & (array[(4+2)%5]));

        let y = 1*5;
        array[0] = a[y + 0];
        array[1] = a[y + 1];
        array[2] = a[y + 2];
        array[3] = a[y + 3];
        array[4] = a[y + 4];
        a[y + 0] = array[0] ^ ((!array[(0+1)%5]) & (array[(0+2)%5]));
        a[y + 1] = array[1] ^ ((!array[(1+1)%5]) & (array[(1+2)%5]));
        a[y + 2] = array[2] ^ ((!array[(2+1)%5]) & (array[(2+2)%5]));
        a[y + 3] = array[3] ^ ((!array[(3+1)%5]) & (array[(3+2)%5]));
        a[y + 4] = array[4] ^ ((!array[(4+1)%5]) & (array[(4+2)%5]));

        let y = 2*5;
        array[0] = a[y + 0];
        array[1] = a[y + 1];
        array[2] = a[y + 2];
        array[3] = a[y + 3];
        array[4] = a[y + 4];
        a[y + 0] = array[0] ^ ((!array[(0+1)%5]) & (array[(0+2)%5]));
        a[y + 1] = array[1] ^ ((!array[(1+1)%5]) & (array[(1+2)%5]));
        a[y + 2] = array[2] ^ ((!array[(2+1)%5]) & (array[(2+2)%5]));
        a[y + 3] = array[3] ^ ((!array[(3+1)%5]) & (array[(3+2)%5]));
        a[y + 4] = array[4] ^ ((!array[(4+1)%5]) & (array[(4+2)%5]));

        let y = 3*5;
        array[0] = a[y + 0];
        array[1] = a[y + 1];
        array[2] = a[y + 2];
        array[3] = a[y + 3];
        array[4] = a[y + 4];
        a[y + 0] = array[0] ^ ((!array[(0+1)%5]) & (array[(0+2)%5]));
        a[y + 1] = array[1] ^ ((!array[(1+1)%5]) & (array[(1+2)%5]));
        a[y + 2] = array[2] ^ ((!array[(2+1)%5]) & (array[(2+2)%5]));
        a[y + 3] = array[3] ^ ((!array[(3+1)%5]) & (array[(3+2)%5]));
        a[y + 4] = array[4] ^ ((!array[(4+1)%5]) & (array[(4+2)%5]));

        let y = 4*5;
        array[0] = a[y + 0];
        array[1] = a[y + 1];
        array[2] = a[y + 2];
        array[3] = a[y + 3];
        array[4] = a[y + 4];
        a[y + 0] = array[0] ^ ((!array[(0+1)%5]) & (array[(0+2)%5]));
        a[y + 1] = array[1] ^ ((!array[(1+1)%5]) & (array[(1+2)%5]));
        a[y + 2] = array[2] ^ ((!array[(2+1)%5]) & (array[(2+2)%5]));
        a[y + 3] = array[3] ^ ((!array[(3+1)%5]) & (array[(3+2)%5]));
        a[y + 4] = array[4] ^ ((!array[(4+1)%5]) & (array[(4+2)%5]));

        // Iota
        a[0] ^= RC[i];
    }
}

fn setout(src: &[u8], dst: &mut [u8], len: usize) {
    dst[..len].copy_from_slice(&src[..len]);
}

fn xorin(dst: &mut [u8], src: &[u8]) {
    assert!(dst.len() <= src.len());
    let len = dst.len();
    let mut dst_ptr = dst.as_mut_ptr();
    let mut src_ptr = src.as_ptr();
    for _ in 0..len {
        unsafe {
            *dst_ptr ^= *src_ptr;
            src_ptr = src_ptr.offset(1);
            dst_ptr = dst_ptr.offset(1);
        }
    }
}

// Total number of lanes.
const PLEN: usize = 25;

pub struct Keccak {
    a: [u64; PLEN],
    offset: usize,
    rate: usize,
    delim: u8
}

macro_rules! impl_constructor {
    ($name: ident, $alias: ident, $bits: expr, $delim: expr) => {
        pub fn $name() -> Keccak {
            Keccak::new(200 - $bits/4, $delim)
        }
    }
}

impl Keccak {
    pub fn new(rate: usize, delim: u8) -> Keccak {
        Keccak {
            a: [0; PLEN],
            offset: 0,
            rate: rate,
            delim: delim
        }
    }
    impl_constructor!(new_sha3_256, sha3_256, 256, 0x06);

    fn a_bytes(&self) -> &[u8; PLEN * 8] {
        unsafe { ::core::mem::transmute(&self.a) }
    }

    fn a_mut_bytes(&mut self) -> &mut [u8; PLEN * 8] {
        unsafe { ::core::mem::transmute(&mut self.a) }
    }

    pub fn update(&mut self, input: &[u8]) {
        self.absorb(input);
    }

    pub fn finalize(mut self, output: &mut [u8]) {
        self.pad();

        // apply keccakf
        keccakf(&mut self.a);

        // squeeze output
        self.squeeze(output);
    }

    // Absorb input
    pub fn absorb(&mut self, input: &[u8]) {
        //first foldp
        let mut ip = 0;
        let mut l = input.len();
        let mut rate = self.rate - self.offset;
        let mut offset = self.offset;
        while l >= rate {
            xorin(&mut self.a_mut_bytes()[offset..][..rate], &input[ip..]);
            keccakf(&mut self.a);
            ip += rate;
            l -= rate;
            rate = self.rate;
            offset = 0;
        }

        // Xor in the last block
        xorin(&mut self.a_mut_bytes()[offset..][..l], &input[ip..]);
        self.offset = offset + l;
    }

    pub fn pad(&mut self) {
        let offset = self.offset;
        let rate = self.rate;
        let delim = self.delim;
        let aa = self.a_mut_bytes();
        aa[offset] ^= delim;
        aa[rate - 1] ^= 0x80;
    }

    // squeeze output
    pub fn squeeze(&mut self, output: &mut [u8]) {
        // second foldp
        let mut op = 0;
        let mut l = output.len();
        while l >= self.rate {
            setout(self.a_bytes(), &mut output[op..], self.rate);
            keccakf(&mut self.a);
            op += self.rate;
            l -= self.rate;
        }

        setout(self.a_bytes(), &mut output[op..], l);
    }
}
