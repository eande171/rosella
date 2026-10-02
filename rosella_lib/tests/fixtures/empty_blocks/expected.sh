#!/bin/bash
x=2
if (( x > 1 )); then
    :
else
    printf '%s\n' "no"
fi
nothing() {
    :
}
nothing
if (( x > 1 )); then
    :
fi
printf '%s\n' "ok"
