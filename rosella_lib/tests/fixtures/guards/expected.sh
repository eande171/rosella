#!/bin/bash
clean() {
    local rosella_clean_dir="${1}"
    rm -rf -- "${rosella_clean_dir:?}/build"
}
empty=""
clean "${empty}"
printf '%s\n' "SHOULD NOT PRINT"
