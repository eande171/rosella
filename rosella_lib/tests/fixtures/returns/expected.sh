#!/bin/bash
add() {
    local rosella_add_a="${1}"
    local rosella_add_b="${2}"
    rosella_return=$(( rosella_add_a + rosella_add_b ))
}
label() {
    local rosella_label_n="${1}"
    if (( rosella_label_n == 1 )); then
        rosella_return="one"
        return
    fi
    rosella_return="many: ${rosella_label_n}"
}
first_square_over() {
    local rosella_first_square_over_limit="${1}"
    i=0
    while (( i < 100 )); do
        if (( (i * i) > rosella_first_square_over_limit )); then
            rosella_return=$(( i ))
            return
        fi
        i=$(( i + 1 ))
    done
    rosella_return=-1
}
double_sum() {
    local rosella_double_sum_a="${1}"
    local rosella_double_sum_b="${2}"
    add "${rosella_double_sum_a}" "${rosella_double_sum_b}"
    rosella_return=$(( rosella_return * 2 ))
}
add "1" "2"
total=$(( rosella_return ))
printf '%s\n' "total: ${total}"
double_sum "2" "3"
printf '%s\n' "doubled: ${rosella_return}"
label "1"
rosella_result0="${rosella_return}"
add "1" "1"
rosella_result1="${rosella_return}"
label "${rosella_result1}"
rosella_result2="${rosella_return}"
printf '%s\n' "${rosella_result0} / ${rosella_result2}"
add "5" "5"
if add "${total}" "1"; (( rosella_return == 4 )); then
    printf '%s\n' "four"
fi
n=0
while add "${n}" "0"; (( rosella_return < 3 )); do
    n=$(( n + 1 ))
done
printf '%s\n' "n: ${n}"
first_square_over "50"
printf '%s\n' "first square over 50: ${rosella_return}"
add "1" "2"
rosella_result3="${rosella_return}"
add "3" "4"
rosella_result4="${rosella_return}"
add "${rosella_result3}" "${rosella_result4}"
rosella_result5="${rosella_return}"
nested=$(( rosella_result5 - 1 ))
printf '%s\n' "nested: ${nested}"
