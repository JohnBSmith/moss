
alias moss-test="../target/debug/moss"
# alias moss-test="valgrind ../target/debug/moss"

moss-test test-basic
moss-test test-object-system
moss-test test-import
moss-test test-sudoku
moss-test test-json
moss-test test-la
moss-test test-generators
moss-test test-exceptions
moss-test test-cf
moss-test test-la-inv
moss-test test-long

# moss-test test-la-inv-complex
# too slow in debug mode


