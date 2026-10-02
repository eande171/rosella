#!/bin/bash
stop() {
    i=0
    while (( i < 5 )); do
        if (( i == 2 )); then
            exit 3
        fi
        i=$(( i + 1 ))
    done
}
printf '%s\n' "before"
stop
printf '%s\n' "SHOULD NOT PRINT"
