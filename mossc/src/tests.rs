
use crate::{CmdInfo, compile};
use error::{Error, ErrorKind};

fn test_compile(input: &str, ) -> Result<(), Error> {
    let info = CmdInfo::new();
    compile(input, "", &info)
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Expect {
    Ok, TypeError, UndefinedType
}

static TESTS1: &[(&str, &str, Expect)] = &[
    ("1.01", "let x = 0", Expect::Ok),
    ("1.02", "let x: Int = 0", Expect::Ok),
    ("1.03", "let x: _ = 0", Expect::Ok),
    ("1.04", "let x: String = 0", Expect::TypeError),
    ("1.05", "let x: Integer = 0", Expect::UndefinedType),
    ("1.06", "let x = 1 + 2", Expect::Ok),
    ("1.07", "let x: Int = 1 + 2", Expect::Ok),
    ("1.08", "let x: _ = 1 + 2", Expect::Ok),
    ("1.09", "let x: String = 1 + 2", Expect::TypeError),
    ("1.10", "let a: List[Int] = 0", Expect::TypeError),
    ("1.11", "let a: List[Int] = 1 + 2", Expect::TypeError),
    ("1.12", "let a: List[Int] = [1, 2]", Expect::Ok),
    ("1.13", "let a: List[Int] = []", Expect::Ok),
    ("1.14", "let a: List[List[Int]] = [[1, 2], [3, 4]]", Expect::Ok),
    ("1.15", "let a: List[List[Int]] = [[], [1, 2]]", Expect::Ok),
    ("1.16", "let a: List[List[Int]] = [[1, 2], []]", Expect::Ok),
    ("1.17", "let a: List[List[Int]] = [[], []]", Expect::Ok),
    ("1.18", "let a: List[String] = [1, 2]", Expect::TypeError),
    ("1.19", "let a: List[_] = [1, 2]", Expect::Ok),
    ("1.20", "let a: List[_] = []", Expect::Ok),
    ("1.21", "let a: List[Object] = [true, 1]", Expect::Ok),
    ("1.22", "let a: List[List[_]] = [[1, 2], [3, 4]]", Expect::Ok),
    ("1.23", "let a: List[List[_]] = [[], [1, 2]]", Expect::Ok),
    ("1.24", "let a: List[List[_]] = [[1, 2], []]", Expect::Ok),
    ("1.25", "let a: List[List[_]] = [[], []]", Expect::Ok),
    ("1.26", "let a: List[List[_]] = []", Expect::Ok),
    ("1.27", "let a: List[_] = [[1, 2], [3, 4]]", Expect::Ok),
    ("1.28", "let a: List[_] = [[], [1, 2]]", Expect::Ok),
    ("1.29", "let a: List[_] = [[1, 2], []]", Expect::Ok),
    ("1.30", "let a: List[_] = [[], []]", Expect::Ok),
    ("1.31", "let a: List[Object] = [[1, 2], [true]]", Expect::Ok),
    ("1.32", "let a: List[Object] = [[true], [1, 2]]", Expect::Ok),
    ("1.33", "let a: F[Int] = []", Expect::UndefinedType),
    ("1.34", "let f = |x| x + 1", Expect::Ok),
    ("1.35", "let f: Int -> Int = |x| x + 1", Expect::Ok),
    ("1.36", "let f: _ -> Int = |x| x + 1", Expect::Ok),
    ("1.37", "let f: Int -> _ = |x| x + 1", Expect::Ok),
    ("1.38", "let f: _ -> _ = |x| x + 1", Expect::Ok),
    ("1.39", "let f = |x: Int| x + 1", Expect::Ok),
    ("1.40", "let f = |x| [x, x]", Expect::Ok),
    ("1.41", "let f: Int -> List[Int] = |x| [x, x]", Expect::Ok),
    ("1.42", "let f: _ -> List[Int] = |x| [x, x]", Expect::Ok),
    ("1.43", "let f: Int -> _ = |x| [x, x]", Expect::Ok),
    ("1.44", "let f: _ -> _ = |x| [x, x]", Expect::Ok),
    ("1.45", "let f: Int -> List[_] = |x| [x, x]", Expect::Ok),
    ("1.46", "let f: _ -> List[_] = |x| [x, x]", Expect::Ok),
    ("1.47", "let f = |x: Int| [x, x]", Expect::Ok),
    ("1.48", "let f = || [1, 2]", Expect::Ok),
    ("1.49", "let f: () -> List[Int] = || [1, 2]", Expect::Ok),
    ("1.50", "let f: () -> List[_] = || [1, 2]", Expect::Ok),
    ("1.51", "let f: () -> _ = || [1, 2]", Expect::Ok),
    ("1.52", "let f: _ -> List[Int]  = || [1, 2]", Expect::TypeError),
    ("1.53", "let f: _ -> List[_] = || [1, 2]", Expect::TypeError),
    ("1.54", "let f: _ -> _ = || [1, 2]", Expect::TypeError),
    ("1.55", "let f = || []", Expect::Ok),
    ("1.56", "let f: () -> List[Int] = || []", Expect::Ok),
    ("1.57", "let f: () -> List[String] = || []", Expect::Ok),
    ("1.58", "let f: () -> List[_] = || []", Expect::Ok),
    ("1.59", "let f: () -> _ = || []", Expect::Ok),
    ("1.60", "let f = || [1, true]", Expect::Ok),
    ("1.61", "let f: () -> List[Int] = || [1, true]", Expect::TypeError),
    ("1.62", "let f: () -> List[Int] = || [true, true]", Expect::TypeError),
    ("1.63", "let f: () -> Int = || []", Expect::TypeError),
    ("1.64", "let f: () -> List[List[Int]] = || []", Expect::Ok),
    ("1.65", "let f: () -> List[List[Int]] = || [[1, 2]]", Expect::Ok),
    ("1.66", "let f: () -> List[List[Int]] = || [[]]", Expect::Ok),
    ("1.67", "let f: () -> List[List[_]] = || []", Expect::Ok),
    ("1.68", "let f: () -> List[List[_]] = || [[1, 2]]", Expect::Ok),
    ("1.69", "let f: () -> List[List[_]] = || [[]]", Expect::Ok),
    ("1.70", "let f: () -> List[_] = || [[1, 2]]", Expect::Ok),
    ("1.71", "let f: () -> List[_] = || [[]]", Expect::Ok),
    ("1.72", "let f: () -> List[Int] = || [1 + 2]", Expect::Ok),
    ("1.73", "let f: () -> List[_] = || [1 + 2]", Expect::Ok),
    ("1.74", "let f: () -> _ = || [1 + 2]", Expect::Ok),
    ("1.75", "let f: () -> List[List[Int]] = || [[1 + 2]]", Expect::Ok),
    ("1.76", "let f: () -> List[List[_]] = || [[1 + 2]]", Expect::Ok),
    ("1.77", "let f: () -> List[_] = || [[1 + 2]]", Expect::Ok),
    ("1.78", "let f: () -> _ = || [[1 + 2]]", Expect::Ok),
    ("1.79", "let f: (Int, Int) -> List[Int] = |x, y| [x, y]", Expect::Ok),
    ("1.80", "let f: (Int, Int) -> List[_] = |x, y| [x, y]", Expect::Ok),
    ("1.81", "let f: (Int, Int) -> _ = |x, y| [x, y]", Expect::Ok),
    ("1.82", "let f: (_, Int) -> _ = |x, y| [x, y]", Expect::Ok),
    ("1.83", "let f: (Int, _) -> _ = |x, y| [x, y]", Expect::Ok),
    ("1.84", "let f: (_, _) -> _ = |x, y| [x, y]", Expect::Ok),
    ("1.85", "let f = |x, y| [x, y]", Expect::Ok),
    ("1.86", "let f = |x: Int, y| [x, y]", Expect::Ok),
    ("1.87", "let f = |x, y: Int| [x, y]", Expect::Ok),
    ("1.88", "let f = |x: Int, y: Int| [x, y]", Expect::Ok),
    ("1.89", "let f = |x: _, y: Int| [x, y]", Expect::Ok),
    ("1.90", "let f = |x: Int, y: _| [x, y]", Expect::Ok),
    ("1.91", "let f = |x: _, y: _| [x, y]", Expect::Ok),
    ("1.92", "let f = |x, y| [] ", Expect::Ok),
    ("1.93", "let f: (Int, Int) -> List[Int] = |x, y| []", Expect::Ok),
    ("1.94", "let f: (_, _) -> List[Int] = |x, y| []", Expect::Ok),
    ("1.95", "let f: (_, _) -> List[_] = |x, y| []", Expect::Ok),
    ("1.96", "let f: (_, _) -> _ = |x, y| []", Expect::Ok),
    ("1.97", "let f: (Int, _) -> _ = |x, y| []", Expect::Ok),
    ("1.98", "let f: (_, Int) -> _ = |x, y| []", Expect::Ok),
    ("1.99", "let f: (Int, Bool) -> _ = |x, y| []", Expect::Ok)
];

static TESTS2: &[(&str, &str, Expect)] = &[
    ("2.01", "let f: (Int, Int) -> Int = |x, y| x + y", Expect::Ok),
    ("2.02", "let f: (Int, Int) -> Bool = |x, y| x + y", Expect::TypeError),
    ("2.03", "let f: (Bool, Int) -> Int = |x, y| x + y", Expect::TypeError),
    ("2.04", "let f: (Int, Bool) -> Int = |x, y| x + y", Expect::TypeError),
    ("2.05", "let f: (Int, Int) -> _ = |x, y| x + y", Expect::Ok),
    ("2.06", "let f: (Int, _) -> Int = |x, y| x + y", Expect::Ok),
    ("2.07", "let f: (_, Int) -> Int = |x, y| x + y", Expect::Ok),
    ("2.08", "let f: (_, _) -> Int = |x, y| x + y", Expect::Ok),
    ("2.09", "let f: (Int, _) -> _ = |x, y| x + y", Expect::Ok),
    ("2.10", "let f: (_, Int) -> _ = |x, y| x + y", Expect::Ok),
    ("2.11", "let f = |x: Int, y: Int| x + y", Expect::Ok),
    ("2.12", "let f = |x: Int, y| x + y", Expect::Ok),
    ("2.13", "let f = |x, y: Int| x + y", Expect::Ok),
    ("2.14", "let f = |x: Int, y: _| x + y", Expect::Ok),
    ("2.15", "let f = |x: _, y: Int| x + y", Expect::Ok),
    ("2.16", "let f = |x, y| x + y", Expect::Ok),
    ("2.17", "let f: (_, _) -> _ = |x, y| x + y", Expect::Ok)
];

static TESTS3: &[(&str, &str, Expect)] = &[
    ("3.01", "let id[T]: T -> T = |x| x", Expect::Ok),
    ("3.02", "let id[T]: T -> _ = |x| x", Expect::Ok),
    ("3.03", "let id[T]: _ -> T = |x| x", Expect::Ok),
    ("3.04", "let id[T] = |x: T| x", Expect::Ok),
    ("3.05", "let id[T]: T -> T = |x: T| x", Expect::Ok),
    ("3.06", "let f[T] = |x: T| [x, x]", Expect::Ok),
    ("3.07", "let f[T]: T -> List[T] = |x| [x, x]", Expect::Ok),
    ("3.08", "let f[T]: T -> List[T] = |x: T| [x, x]", Expect::Ok),
    ("3.09", "let f[T: Add] = |x: T, y: T| x + y", Expect::Ok),
    ("3.10", "let f[T: Add] = |x: T, y| x + y", Expect::Ok),
    ("3.11", "let f[T: Add] = |x, y: T| x + y", Expect::Ok),
    ("3.12", "let f[T: Add]: (T, T) -> T = |x, y| x + y", Expect::Ok),
    ("3.13", "let f[T: Add]: (T, T) -> _ = |x, y| x + y", Expect::Ok),
    ("3.14", "let f[T: Add]: (T, _) -> _ = |x, y| x + y", Expect::Ok),
    ("3.15", "let f[T: Add]: (_, T) -> _ = |x, y| x + y", Expect::Ok),
    ("3.16", "let f[T: Add]: (_, _) -> T = |x, y| x + y", Expect::Ok),
    ("3.17", "let f[T] = |x: T, y: T| x + y", Expect::TypeError),
    ("3.18", "let f[T] = |x: T, y| x + y", Expect::TypeError),
    ("3.19", "let f[T] = |x, y: T| x + y", Expect::TypeError),
    ("3.20", "let f[T]: (T, T) -> T = |x, y| x + y", Expect::TypeError),
    ("3.21", "let f[T]: (T, _) -> _ = |x, y| x + y", Expect::TypeError),
    ("3.22", "let f[T]: (_, T) -> _ = |x, y| x + y", Expect::TypeError),
    ("3.23", "let f[T]: (_, _) -> T = |x, y| x + y", Expect::TypeError),
    ("3.24", "let f[T: Add + Sub] = |x: T, y: T| [x + y, x - y]", Expect::Ok),
    ("3.25", "let f[T: Add + Sub] = |x: T, y| [x + y, x - y]", Expect::Ok),
    ("3.26", "let f[T: Add + Sub] = |x, y: T| [x + y, x - y]", Expect::Ok),
    ("3.27", "let f[T: Add + Sub]: (T, T) -> List[T] = (|x, y|
        [x + y, x - y])", Expect::Ok),
    ("3.28", "let f[T: Add + Sub]: (T, _) -> _ = (|x, y|
        [x + y, x - y])", Expect::Ok),
    ("3.29", "let f[T: Add + Sub]: (_, T) -> _ = (|x, y|
        [x + y, x - y])", Expect::Ok),
    ("3.30", "let f[T: Add + Sub]: (_, _) -> List[T] = (|x, y|
        [x + y, x - y])", Expect::Ok),
    ("3.31", "let f[T: Add] = |x: T| x + x; f([1, 2])", Expect::Ok),
    ("3.32", "let f[T: Sub] = |x: T| x - x; f([1, 2])", Expect::TypeError),
    ("3.33", "let f[T: Add] = |x: T| x + x; f([])", Expect::TypeError),
    ("3.34", "let f[T: Sub] = |x: T| x - x; f([])", Expect::TypeError),
    ("3.35", "let f = |x, y| x == y; f(0, 0)", Expect::Ok),
    ("3.36", "let f = |x, y| [x] == [y]; f(0, 0)", Expect::Ok),
    ("3.37", "let f = |x, y| [[x]] == [[y]]; f(0, 0)", Expect::Ok),
    ("3.38", "[] == []", Expect::TypeError),
    ("3.39", "[0] == []", Expect::Ok),
    ("3.40", "[] == [0]", Expect::Ok)
];

static TESTS4: &[(&str, &str, Expect)] = &[
    ("4.01", "[1, 2].map(|x| 2*x)", Expect::Ok),
    ("4.02", "[true, true].map(|x| 2*x)", Expect::TypeError),
    ("4.03", "[true, true].map(|x| x*x)", Expect::TypeError)
];

fn is_expected(result: &Result<(), Error>, expected: Expect) -> bool {
    expected == match result {
        Ok(()) => Expect::Ok,
        Err(e) => match e.kind {
            ErrorKind::TypeError => Expect::TypeError,
            ErrorKind::UndefinedType => Expect::UndefinedType,
            _ => return false
        }
    }
}

fn test_list(tests: &[(&str, &str, Expect)]) {
    for (id, input, expected) in tests {
        let result = test_compile(input);
        if !is_expected(&result, *expected) {
            if let Err(e) = result {
                println!("{}", e);
            }
            panic!("compiler test {} failed", id);
        }
    }
}

#[test]
fn test0() {
    test_list(TESTS1);
    test_list(TESTS2);
    test_list(TESTS3);
    test_list(TESTS4);
}
