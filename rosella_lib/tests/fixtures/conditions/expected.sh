#!/bin/bash
x=2
if (( x == 1 )); then
    printf '%s\n' "one"
elif (( (x + 1) == 3 )); then
    printf '%s\n' "two"
else
    printf '%s\n' "other"
fi
name="Bob"
if [[ "${name}" != "Alice" ]]; then
    printf '%s\n' "not Alice"
else
    printf '%s\n' "Alice"
fi
if [[ "${name}" == "Bob" ]]; then
    printf '%s\n' "Bob"
fi
if [[ "apple" < "banana" ]]; then
    printf '%s\n' "apple first"
fi
if [[ ! -e "missing.txt" ]]; then
    printf '%s\n' "no file"
fi
