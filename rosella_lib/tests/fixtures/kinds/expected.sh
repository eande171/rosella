#!/bin/bash
mkdir -p -- "full"
printf '%s\n' "x" > "full/inside.txt"
printf '%s\n' "x" > "plain.txt"
if [[ ! -d "full" ]]; then rm -f -- "full"; fi
if [[ -f "full/inside.txt" ]]; then
    printf '%s\n' "remove left the folder alone"
fi
if [[ -d "plain.txt" ]]; then rm -rf -- "plain.txt"; fi
if [[ -f "plain.txt" ]]; then
    printf '%s\n' "remove_dir left the file alone"
fi
cp -- "full" "full_copy"
if [[ ! -e "full_copy" ]]; then
    printf '%s\n' "copy skipped the folder"
fi
mv -- "full" "renamed"
if [[ -f "renamed/inside.txt" ]] && [[ ! -e "full" ]]; then
    printf '%s\n' "move renamed the folder"
fi
mv -- "plain.txt" "renamed"
if [[ -f "renamed/plain.txt" ]]; then
    printf '%s\n' "move put the file in the folder"
fi
if [[ -d "renamed" ]]; then rm -rf -- "renamed"; fi
command "git" "config" "--file" "none.cfg" "--get" "x.y"
exit 0
