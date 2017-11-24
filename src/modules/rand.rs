
const KX: u32 = 123456789;
const KY: u32 = 234567891;
const KZ: u32 = 345678912;
const KW: u32 = 456789123;
const KC: u32 = 0;

pub struct Rand {
  x: u32, y: u32, z: u32,
  w: u32, c: u32
}
impl Rand{
  pub fn new(seed: u32) -> Rand {
    let seed = seed.wrapping_add(1);
    Rand{
      x: KX.wrapping_mul(seed),
      y: KY.wrapping_mul(seed),
      z: KZ, w: KW, c: KC
    }
  }

  // This is called "JKISS32", a modification of George Marsaglias KISS.
  // It should pass all tests of TestU01, including BigCrush.
  pub fn rand(&mut self) -> u32 {
    self.y ^= self.y.wrapping_shl(5);
    self.y ^= self.y.wrapping_shr(7);
    self.y ^= self.y.wrapping_shl(22);
    let t = self.z.wrapping_add(self.w).wrapping_add(self.c);
    self.z = self.w;
    self.c = (t>0x7fffffff) as u32;
    self.w = t&0x7fffffff;
    self.x = self.x.wrapping_add(1411392427);
    return self.x.wrapping_add(self.y).wrapping_add(self.w);
  }

  pub fn shuffle<T>(&mut self, a: &mut [T]) {
    let mut i=a.len()-1;
    while i>0 {
      let j=(self.rand() as usize)%(i+1);
      a.swap(i,j);
      i-=1;
    }
  }
}

