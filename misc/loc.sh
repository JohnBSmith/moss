
cd ../src
printf "Rust: "
find -name "*.rs" | xargs cat | wc -l

cd ../lib
printf "Moss: "
find -name "*.moss" | xargs cat | wc -l
