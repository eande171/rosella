#!/bin/bash
x=$(( (1 + 2) * 3 ))
y=$(( x - (-4) ))
printf '%s\n' "${x} ${y}"
show() {
    local n="${1}"
    printf '%s\n' "n=${n}"
}
show "$(( x * 3 ))"
show "$(( (x + 1) * (-2) ))"
