#!/bin/bash
clean() {
    local rosella_clean_file="${1}"
    if [[ ! -d "data/${rosella_clean_file:?}" ]]; then rm -f -- "data/${rosella_clean_file:?}"; fi
}
mkdir -p -- "data"
printf '%s\n' "x" > "data/keep.txt"
pattern="*.txt"
clean "${pattern}"
if [[ -e "data/${pattern}" ]]; then
    printf '%s\n' "SHOULD NOT PRINT"
fi
if [[ ! -e "data/${pattern}" ]]; then
    printf '%s\n' "a wildcard in a variable matches nothing"
fi
found=0
for file in "data/""${pattern}"; do
    [[ -f "${file}" ]] || continue
    found=$(( found + 1 ))
done
printf '%s\n' "files() with a wildcard in a variable found ${found}"
if [[ -e "data/keep.txt" ]]; then
    printf '%s\n' "keep.txt survived"
fi
if [[ -d "data" ]]; then rm -rf -- "data"; fi
exit 0
