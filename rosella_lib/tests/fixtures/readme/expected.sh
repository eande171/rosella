#!/bin/bash
add() {
    local rosella_add_x="${1}"
    local rosella_add_y="${2}"
    result=$(( rosella_add_x + rosella_add_y ))
    printf '%s\n' "Result: ${result}"
}
add "1" "2"
add "3" "4"
add "5" "6"
x=0
while (( x < 100 )); do
    printf '%s\n' "Current value of x: ${x}"
    x=$(( x + 1 ))
    printf '%s\n' "secret_index_${x}"
done
exit 0
