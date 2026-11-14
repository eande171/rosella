#!/bin/bash
rosella_args=("$@")
rosella_result0=${#rosella_args[@]}
printf '%s\n' "count: ${rosella_result0}"
rosella_result1="${rosella_args[0]}"
printf '%s\n' "first: ${rosella_result1}"
rosella_result2="${rosella_args[1]}"
printf '%s\n' "second: ${rosella_result2}"
rosella_result3="${rosella_args[2]}"
printf '%s\n' "missing: [${rosella_result3}]"
i=1
while rosella_result4=${#rosella_args[@]}; (( i <= rosella_result4 )); do
    rosella_result5=""
    if (( i >= 1 )); then rosella_result5="${rosella_args[i - 1]}"; fi
    printf '%s\n' "${i}: ${rosella_result5}"
    i=$(( i + 1 ))
done
export FIXTURE_MODE="50% & (fast)!"
printf '%s\n' "mode: ${FIXTURE_MODE}"
if [[ "${FIXTURE_UNSET}" == "" ]]; then
    printf '%s\n' "unset is empty"
fi
printf '%s\n' "same" > "50% & (more)!.txt"
printf '%s\n' "same" > "same.txt"
printf '%s\n' "different" > "other.txt"
command "git" "diff" "--no-index" "--quiet" "50% & (more)!.txt" "same.txt"
same=$?
command "git" "diff" "--no-index" "--quiet" "same.txt" "other.txt"
different=$?
printf '%s\n' "same: ${same} different: ${different}"
if rosella_result7="${rosella_args[1]}"; command "git" "diff" "--no-index" "--quiet" "${rosella_result7}" "same.txt"; rosella_result6=$?; (( rosella_result6 == 0 )); then
    printf '%s\n' "argument reached git intact"
fi
command "git" "diff" "--no-index" "--quiet" "same.txt" "same.txt"
if command "rosella_missing_command"; rosella_result8=$?; (( rosella_result8 != 0 )); then
    printf '%s\n' "missing command failed"
fi
rosella_result9=$(( (same + different) + 1 ))
printf '%s\n' "sum: ${rosella_result9}"
exit 0
