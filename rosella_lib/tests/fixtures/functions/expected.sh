#!/bin/bash
greet() {
    local rosella_greet_name="${1}"
    local rosella_greet_greeting="${2}"
    printf '%s\n' "${rosella_greet_greeting} ${rosella_greet_name}."
}
bump() {
    local rosella_bump_x="${1}"
    rosella_bump_x=$(( rosella_bump_x + 100 ))
    printf '%s\n' "inside: ${rosella_bump_x}"
}
only_if() {
    printf '%s\n' "defined inside an if"
}
later() {
    local rosella_later_message="${1}"
    printf '%s\n' "${rosella_later_message}"
}
a_b() {
    local rosella_a_b_c="${1}"
    a "5"
    printf '%s\n' "a_b still has c = ${rosella_a_b_c}"
}
a() {
    local rosella_a_b_c="${1}"
    printf '%s\n' "a got ${rosella_a_b_c}"
}
show_x() {
    printf '%s\n' "show_x sees ${x}"
}
shadow() {
    local rosella_shadow_x="${1}"
    show_x
}
nothing() {
    :
}
greet "Bob Smith" "Hi!"
n="Ann & Lee (100%)"
greet "${n}" "Hello"
x=5
bump "1"
printf '%s\n' "outside: ${x}"
later "called before its definition"
if (( x == 99 )); then
    :
fi
only_if
a_b "1"
shadow "99"
nothing
printf '%s\n' "nothing returned"
exit 0
