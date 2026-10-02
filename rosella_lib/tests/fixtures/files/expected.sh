#!/bin/bash
project="proj"
mkdir -p -- "${project}/build/out"
mkdir -p -- "${project}/build/out"
printf '%s\n' "theme=dark"$'\n'"name=${project}!" > "${project}/settings.ini"
printf '%s\n' "more=100%" >> "${project}/settings.ini"
cp -- "${project}/settings.ini" "${project}/build/copy.ini"
mv -- "${project}/build/copy.ini" "${project}/moved.ini"
if [[ -e "${project}/moved.ini" ]]; then
    printf '%s\n' "moved"
fi
if [[ ! -e "${project}/build/copy.ini" ]]; then
    printf '%s\n' "copy gone"
fi
rm -f -- "${project:?}/moved.ini"
rm -rf -- "${project:?}/build"
if [[ ! -e "${project}/build" ]]; then
    printf '%s\n' "build removed"
fi
if (( 1 == 1 )); then
    cd "${project}"
    here="${PWD}"
    if [[ -e "${here}/settings.ini" ]]; then
        printf '%s\n' "in project"
    fi
fi
cat settings.ini
cd ".."
rm -rf -- "${project:?}"
