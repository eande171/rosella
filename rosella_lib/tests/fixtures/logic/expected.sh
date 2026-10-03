#!/bin/bash
noisy() {
    local rosella_noisy_value="${1}"
    printf '%s\n' "noisy ran with ${rosella_noisy_value}"
    rosella_return=$(( rosella_noisy_value ))
}
x=2
name="Bob"
if (( x > 0 )) && [[ "${name}" == "Bob" ]]; then
    printf '%s\n' "and: yes"
fi
if (( x > 5 )) || [[ "${name}" == "Bob" ]]; then
    printf '%s\n' "or: yes"
fi
if ! { (( x == 1 )) || (( x == 3 )); }; then
    printf '%s\n' "not: yes"
fi
if ! [[ -e "missing.txt" ]] && (( x == 2 )); then
    printf '%s\n' "file and int: yes"
fi
if (( x == 9 )) || { (( x == 2 )) && [[ "${name}" == "Bob" ]]; }; then
    printf '%s\n' "and binds tighter"
fi
if ! { ! (( x == 2 )); }; then
    printf '%s\n' "double not"
fi
if (( x == 9 )) && { noisy "1"; (( rosella_return == 1 )); }; then
    printf '%s\n' "SHOULD NOT PRINT"
fi
if (( x == 2 )) || { noisy "2"; (( rosella_return == 2 )); }; then
    printf '%s\n' "or stopped early"
fi
if (( x == 2 )) && { noisy "3"; (( rosella_return == 3 )); }; then
    printf '%s\n' "and ran both"
fi
rosella_result0=$(( 10 % 3 ))
rosella_result1=$(( 17 % 5 ))
printf '%s\n' "10 % 3 = ${rosella_result0}, 17 % 5 = ${rosella_result1}"
r=$(( x % 2 ))
printf '%s\n' "remainder: ${r}"
i=0
while (( i < 10 )); do
    i=$(( i + 1 ))
    if (( (i % 2) == 0 )); then
        continue
    fi
    if (( i == 7 )); then
        break
    fi
    printf '%s\n' "odd: ${i}"
done
outer=0
while (( outer < 2 )); do
    inner=0
    while (( 1 == 1 )); do
        inner=$(( inner + 1 ))
        if (( inner > 2 )); then
            break
        fi
    done
    printf '%s\n' "outer ${outer} inner ${inner}"
    outer=$(( outer + 1 ))
done
n=0
while (( n < 5 )) && { noisy "${n}"; (( rosella_return != 3 )); }; do
    n=$(( n + 1 ))
done
printf '%s\n' "stopped at ${n}"
exit 0
