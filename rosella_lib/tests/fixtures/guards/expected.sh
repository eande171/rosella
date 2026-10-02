#!/bin/bash
empty=""
clean() {
    local dir="${1}"
    rm -rf -- "${dir:?}/build"
}
clean "${empty}"
printf '%s\n' "SHOULD NOT PRINT"
