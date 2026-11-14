#!/bin/bash
dot() {
    rosella_return="."
}
other() {
    rosella_return="other"
}
first_log() {
    for file in "data/"*".log"; do
        [[ -f "${file}" ]] || continue
        rosella_return="${file}"
        return
    done
    rosella_return="none"
}
square_root() {
    local rosella_square_root_target="${1}"
    for (( n = 1; n < 100; n += 1 )); do
        if (( (n * n) == rosella_square_root_target )); then
            rosella_return=$(( n ))
            return
        fi
    done
    rosella_return=-1
}
for (( i = 0; i < 5; i += 1 )); do
    if (( i == 1 )); then
        continue
    fi
    if (( i == 4 )); then
        break
    fi
    printf '%s\n' "up: ${i}"
done
for (( i = 10; i > 0; i -= 3 )); do
    printf '%s\n' "down: ${i}"
done
limit=3
rosella_result0=$(( limit ))
for (( k = 0; k < rosella_result0; k += 1 )); do
    limit=$(( limit + 10 ))
    printf '%s\n' "k: ${k}"
done
printf '%s\n' "limit is now ${limit}"
mkdir -p -- "data"
mkdir -p -- "data/folder.txt"
printf '%s\n' "second" > "data/b.txt"
printf '%s\n' "first" > "data/a.txt"
printf '%s\n' "other" > "data/c.log"
printf '%s\n' "skipped like a hidden file" > "data/.hidden.txt"
count=0
for file in "data/"*".txt"; do
    [[ -f "${file}" ]] || continue
    count=$(( count + 1 ))
    if [[ "${file}" == "data/a.txt" ]]; then
        printf '%s\n' "found a.txt"
    elif [[ "${file}" == "data/b.txt" ]]; then
        printf '%s\n' "found b.txt"
    else
        printf '%s\n' "UNEXPECTED MATCH"
    fi
done
printf '%s\n' "txt files: ${count}"
for file in "data/"*".txt"; do
    [[ -f "${file}" ]] || continue
    if [[ "${file}" == "data/a.txt" ]]; then
        continue
    fi
    printf '%s\n' "after continue"
done
for file in "data/"*".txt"; do
    [[ -f "${file}" ]] || continue
    printf '%s\n' "before break"
    break
done
for file in "data/"*".none"; do
    [[ -f "${file}" ]] || continue
    printf '%s\n' "SHOULD NOT PRINT"
done
printf '%s\n' "x" > "data/wow!.md"
printf '%s\n' "x" > "data/a^b!c.md"
printf '%s\n' "x" > "data/50% ^ plain.md"
kept=0
for file in "data/"*".md"; do
    [[ -f "${file}" ]] || continue
    if [[ -f "${file}" ]]; then
        kept=$(( kept + 1 ))
    fi
    if [[ "${file}" == "data/wow!.md" ]]; then
        printf '%s\n' "found wow!.md"
    fi
done
printf '%s\n' "names kept: ${kept}"
dotted=0
for file in "data/."*; do
    [[ -f "${file}" ]] || continue
    dotted=$(( dotted + 1 ))
done
printf '%s\n' "dot names: ${dotted}"
printf '%s\n' "x" > "data/.second"
both=0
dot
for file in "data/""${rosella_return}"*; do
    [[ -f "${file}" ]] || continue
    other
    changed="${rosella_return}"
    both=$(( both + 1 ))
done
printf '%s\n' "dot names from a call: ${both}"
if first_log; [[ "${rosella_return}" == "data/c.log" ]]; then
    printf '%s\n' "returned from a files loop"
fi
square_root "49"
printf '%s\n' "square root of 49: ${rosella_return}"
exit 0
