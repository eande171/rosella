#!/bin/bash
x=1
i=0
if (( x == 1 )); then
    while (( i < 2 )); do
        j=0
        while (( j < 2 )); do
            printf '%s\n' "${i}${j}"
            j=$(( j + 1 ))
        done
        i=$(( i + 1 ))
    done
    printf '%s\n' "then"
else
    printf '%s\n' "ELSE SHOULD NOT RUN"
fi
exit 0
