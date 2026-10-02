#!/bin/bash
add() {
    local a="${1}"
    local b="${2}"
    rosella_return=$(( a + b ))
}
label() {
    local n="${1}"
    if (( n == 1 )); then
        rosella_return="one"
        return
    fi
    rosella_return="many: ${n}"
}
first_square_over() {
    local limit="${1}"
    i=0
    while (( i < 100 )); do
        if (( (i * i) > limit )); then
            rosella_return=$(( i ))
            return
        fi
        i=$(( i + 1 ))
    done
    rosella_return=-1
}
double_sum() {
    local a="${1}"
    local b="${2}"
    add "${a}" "${b}"
    local rosella_result0="${rosella_return}"
    rosella_return=$(( rosella_result0 * 2 ))
}
add "1" "2"
total=$(( rosella_return ))
printf '%s\n' "total: ${total}"
double_sum "2" "3"
rosella_result1="${rosella_return}"
printf '%s\n' "doubled: ${rosella_result1}"
label "1"
rosella_result2="${rosella_return}"
add "1" "1"
rosella_result3="${rosella_return}"
label "${rosella_result3}"
rosella_result4="${rosella_return}"
printf '%s\n' "${rosella_result2} / ${rosella_result4}"
add "5" "5"
if add "${total}" "1"; rosella_result5="${rosella_return}"; (( rosella_result5 == 4 )); then
    printf '%s\n' "four"
fi
n=0
while add "${n}" "0"; rosella_result6="${rosella_return}"; (( rosella_result6 < 3 )); do
    n=$(( n + 1 ))
done
printf '%s\n' "n: ${n}"
first_square_over "50"
rosella_result7="${rosella_return}"
printf '%s\n' "first square over 50: ${rosella_result7}"
add "1" "2"
rosella_result8="${rosella_return}"
add "3" "4"
rosella_result9="${rosella_return}"
add "${rosella_result8}" "${rosella_result9}"
rosella_result10="${rosella_return}"
nested=$(( rosella_result10 - 1 ))
printf '%s\n' "nested: ${nested}"
