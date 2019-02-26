
printf "Rust: "
(cd ../src; find -name "*.rs" | xargs cat | wc -l)

printf "  Compiler +"
(cd ../mossc/src; find -name "*.rs" | xargs cat | wc -l)

printf "Moss: "
(cd ../lib; find -name "*.moss" | xargs cat | wc -l)

