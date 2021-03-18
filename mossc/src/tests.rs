
use crate::{CmdInfo, compile};
use error::{Error, ErrorKind};

fn test_compile(input: &str, ) -> Result<(), Error> {
    let info = CmdInfo::new();
    compile(input, "", &info)
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Expect {
    Ok, TypeError
}

static TESTS0: &[(&str, &str, Expect)] = &[
    ("1.00", "let x = 0", Expect::Ok),
    ("1.01", "let x: Int = 0", Expect::Ok),
    ("1.02", "let x: String = 0", Expect::TypeError),
    ("1.03", "let x: Int = 1 + 2", Expect::Ok),
    ("1.04", "let a: List[Int] = [1, 2]", Expect::Ok),
    ("1.05", "let a: List[Int] = []", Expect::Ok),
    ("1.06", "let a: List[List[Int]] = [[1, 2], [3, 4]]", Expect::Ok),
    ("1.07", "let a: List[List[Int]] = [[], []]", Expect::Ok),
    ("1.08", "let a: List[String] = [1, 2]", Expect::TypeError),
    ("1.09", "let a: List[_] = [1, 2]", Expect::Ok),
    ("1.10", "let a: List[_] = []", Expect::Ok),
    ("1.11", "let a: List[_] = [true, 1]", Expect::TypeError)
];

fn is_expected(result: &Result<(), Error>, expected: Expect) -> bool {
    match result {
        Ok(()) => expected == Expect::Ok,
        Err(Error {kind: ErrorKind::TypeError, ..})
        => expected == Expect::TypeError,
        _ => false
    }
}

#[test]
fn test0() {
    for (id, input, expected) in TESTS0 {
        let result = test_compile(input);
        if !is_expected(&result, *expected) {
            panic!("compiler test {} failed", id);
        }
    }
}
