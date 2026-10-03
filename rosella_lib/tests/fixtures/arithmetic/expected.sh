#!/bin/bash
show() {
    local rosella_show_n="${1}"
    printf '%s\n' "n=${rosella_show_n}"
}
x=$(( (1 + 2) * 3 ))
y=$(( x - (-4) ))
printf '%s\n' "${x} ${y}"
show "$(( x * 3 ))"
show "$(( (x + 1) * (-2) ))"
exit 0
