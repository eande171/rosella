#!/bin/bash
greet() {
    local name="${1}"
    local greeting="${2}"
    printf '%s\n' "${greeting} ${name}."
}
greet "Bob Smith" "Hi!"
n="Ann & Lee (100%)"
greet "${n}" "Hello"
x=5
bump() {
    local x="${1}"
    x=$(( x + 100 ))
    printf '%s\n' "inside: ${x}"
}
bump "1"
printf '%s\n' "outside: ${x}"
