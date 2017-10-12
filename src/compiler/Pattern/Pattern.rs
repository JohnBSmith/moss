
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

struct T {}

struct Process {
  state_change: bool
}

struct Iterator<'a> {
  a: &'a [T],
  index: usize,
  list: Vec<Vec<T>>
}

impl<'a> Iterator<'a> {
  fn get(&mut self, p: &'a mut Process) -> &'a T {
    if p.state_change {
      p.proceed_to_new_location(self);
    }
    return &self.a[self.index];
  }
}


impl Process {
  fn proceed_to_new_location<'a>(&'a mut self, i: &mut Iterator<'a>) {
    i.index=0;
    i.list.push(vec![T{},T{}]);
    i.a = &i.list[i.list.len()-1];
  }
}

fn main() {
  let mut v = vec![T{},T{}];
  let mut process = Process{state_change: false};

  let mut i = Iterator{index: 0, a: &v, list: vec![]};
  let t = i.get(&mut process);

  let t = i.get(&mut process);
}


