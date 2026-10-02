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
setting "demo.key"
printf '%s\n' "${rosella_return}"
