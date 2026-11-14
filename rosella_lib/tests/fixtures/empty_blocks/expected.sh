#!/bin/bash
nothing() {
    :
}
x=2
if (( x > 1 )); then
    :
else
    printf '%s\n' "no"
fi
nothing
if (( x > 1 )); then
    :
fi
printf '%s\n' "ok"
exit 0
