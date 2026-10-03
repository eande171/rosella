#!/bin/bash
rosella_script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
name="Hello World"
rosella_result0=${#name}
rosella_text=""
rosella_result1=${#rosella_text}
printf '%s\n' "length: ${rosella_result0} empty: ${rosella_result1}"
rosella_result2="${name:0:5}"
rosella_result3="${name:6:100}"
rosella_result4="${name:(-5):3}"
printf '%s\n' "slice: ${rosella_result2}|${rosella_result3}|${rosella_result4}"
rosella_text="abc"
rosella_result5=${#rosella_text}
for (( i = 0; i < rosella_result5; i += 1 )); do
    rosella_text="abc"
    rosella_result6="${rosella_text:i:1}"
    printf '%s\n' "char ${i}: ${rosella_result6}"
done
rosella_result7="${name//"o"/"0"}"
printf '%s\n' "replace: ${rosella_result7}"
rosella_text="aAaA"
rosella_result8="${rosella_text//"a"/"x"}"
printf '%s\n' "case: ${rosella_result8}"
rosella_text="one fish two fish"
rosella_result9="${rosella_text//"fish"/"cat"}"
printf '%s\n' "words: ${rosella_result9}"
rosella_text="50% & (x)"
rosella_result10="${rosella_text//"&"/"and"}"
printf '%s\n' "special: ${rosella_result10}"
rosella_result11="${name^^[a-z]}"
rosella_result12="${name,,[A-Z]}"
printf '%s\n' "upper: ${rosella_result11} lower: ${rosella_result12}"
rosella_text="café"
rosella_result13="${rosella_text^^[a-z]}"
rosella_text="CAFÉ"
rosella_result14="${rosella_text,,[A-Z]}"
printf '%s\n' "accents stay: ${rosella_result13} ${rosella_result14}"
if [[ "${name}" == *"World"* ]]; then
    printf '%s\n' "contains World"
fi
if ! [[ "${name}" == *"world"* ]]; then
    printf '%s\n' "case matters"
fi
if [[ "${name}" == *""* ]]; then
    printf '%s\n' "empty text is contained"
fi
roll=$(( 1 + RANDOM % (6 - 1 + 1) ))
if (( roll >= 1 )) && (( roll <= 6 )); then
    printf '%s\n' "roll in range"
fi
rosella_result15=$(( 4 + RANDOM % (4 - 4 + 1) ))
printf '%s\n' "fixed: ${rosella_result15}"
mkdir -p -- "folder"
printf '%s\n' "x" > "note.txt"
if [[ -d "folder" ]] && ! [[ -f "folder" ]]; then
    printf '%s\n' "folder is a folder"
fi
if [[ -f "note.txt" ]] && ! [[ -d "note.txt" ]]; then
    printf '%s\n' "note is a file"
fi
if ! [[ -d "missing" ]] && ! [[ -f "missing" ]]; then
    printf '%s\n' "missing is neither"
fi
if [[ -f "${rosella_script_dir}/expected.sh" ]] || [[ -f "${rosella_script_dir}/expected.bat" ]]; then
    printf '%s\n' "script_dir found the script"
fi
nothing=""
if ! [[ -d "${nothing}" ]] && ! [[ -f "${nothing}" ]] && ! [[ -e "${nothing}" ]]; then
    printf '%s\n' "an empty path is nothing"
fi
if [[ -f "missing" ]]; then
    printf '%s\n' "SHOULD NOT PRINT"
else
    printf '%s\n' "is_file keeps its else"
fi
sleep 1
printf '%s\n' "slept"
exit 0
