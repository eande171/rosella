#!/bin/bash
setting() {
    local rosella_setting_key="${1}"
    rosella_line=""
    rosella_return=""
    while IFS= read -r rosella_line; do rosella_line="${rosella_line%$'\r'}"; if [[ -n "${rosella_line}" ]]; then rosella_return="${rosella_line}"; break; fi; done < <(command "git" "-c" "${rosella_setting_key}=from a function" "config" "--get" "${rosella_setting_key}")
}
rosella_line=""
name=""
while IFS= read -r rosella_line; do rosella_line="${rosella_line%$'\r'}"; if [[ -n "${rosella_line}" ]]; then name="${rosella_line}"; break; fi; done < <(command "git" "-c" "fixture.name=Hello & 50% (done)" "config" "--get" "fixture.name")
printf '%s\n' "name: ${name}"
rosella_line=""
first=""
while IFS= read -r rosella_line; do rosella_line="${rosella_line%$'\r'}"; if [[ -n "${rosella_line}" ]]; then first="${rosella_line}"; break; fi; done < <(command "git" "-c" "a.b=1" "-c" "a.c=2" "config" "--get-regexp" "^a[.]")
printf '%s\n' "first line: ${first}"
if rosella_line=""; rosella_result0=""; while IFS= read -r rosella_line; do rosella_line="${rosella_line%$'\r'}"; if [[ -n "${rosella_line}" ]]; then rosella_result0="${rosella_line}"; break; fi; done < <(command "git" "config" "--file" "missing.cfg" "--get" "x.y"); [[ "${rosella_result0}" == "" ]]; then
    printf '%s\n' "empty output"
fi
rosella_line=""
rosella_result1=""
while IFS= read -r rosella_line; do rosella_line="${rosella_line%$'\r'}"; if [[ -n "${rosella_line}" ]]; then rosella_result1="${rosella_line}"; break; fi; done < <(command "git" "-c" "x.y=inline" "config" "--get" "x.y")
printf '%s\n' "inline: ${rosella_result1}"
rosella_line=""
rosella_result2=""
while IFS= read -r rosella_line; do rosella_line="${rosella_line%$'\r'}"; if [[ -n "${rosella_line}" ]]; then rosella_result2="${rosella_line}"; break; fi; done < <(command "git" "-c" "x.y=Done! 100% ^ caret" "config" "--get" "x.y")
printf '%s\n' "special: ${rosella_result2}"
rosella_line=""
rosella_result3=""
while IFS= read -r rosella_line; do rosella_line="${rosella_line%$'\r'}"; if [[ -n "${rosella_line}" ]]; then rosella_result3="${rosella_line}"; break; fi; done < <(command "git" "-c" "x.y=a ^ b" "config" "--get" "x.y")
printf '%s\n' "plain caret: ${rosella_result3}"
rosella_line=""
rosella_result4=""
while IFS= read -r rosella_line; do rosella_line="${rosella_line%$'\r'}"; if [[ -n "${rosella_line}" ]]; then rosella_result4="${rosella_line}"; break; fi; done < <(command "git" "-c" "x.y=%PATH% stays" "config" "--get" "x.y")
printf '%s\n' "percent name: ${rosella_result4}"
rosella_line=""
rosella_result5=""
while IFS= read -r rosella_line; do rosella_line="${rosella_line%$'\r'}"; if [[ -n "${rosella_line}" ]]; then rosella_result5="${rosella_line}"; break; fi; done < <(command "git" "-c" "x.y=C:\\dir\\" "config" "--get" "x.y")
printf '%s\n' "backslash: ${rosella_result5}"
folder="C:\\data\\\\"
rosella_line=""
rosella_result6=""
while IFS= read -r rosella_line; do rosella_line="${rosella_line%$'\r'}"; if [[ -n "${rosella_line}" ]]; then rosella_result6="${rosella_line}"; break; fi; done < <(command "git" "-c" "x.y=${folder}" "config" "--get" "x.y")
printf '%s\n' "backslashes: ${rosella_result6}"
command "git" "-c" "x.y=run ${folder}" "config" "--get" "x.y"
printf '%s\n' "[x]"$'\n'"	z = plain \\\" & echo INJECTED & rem \\\""$'\n'"	m ="$'\n'"	m = second" > "q.cfg"
rosella_line=""
rosella_result7=""
while IFS= read -r rosella_line; do rosella_line="${rosella_line%$'\r'}"; if [[ -n "${rosella_line}" ]]; then rosella_result7="${rosella_line}"; break; fi; done < <(command "git" "config" "--file" "q.cfg" "--get" "x.z")
printf '%s\n' "quotes: ${rosella_result7}"
rosella_line=""
rosella_result8=""
while IFS= read -r rosella_line; do rosella_line="${rosella_line%$'\r'}"; if [[ -n "${rosella_line}" ]]; then rosella_result8="${rosella_line}"; break; fi; done < <(command "git" "config" "--file" "q.cfg" "--get-all" "x.m")
printf '%s\n' "first line with text: [${rosella_result8}]"
setting "demo.key"
printf '%s\n' "${rosella_return}"
exit 0
