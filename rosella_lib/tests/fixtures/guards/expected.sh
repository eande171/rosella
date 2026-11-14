#!/bin/bash
clean() {
    local rosella_clean_dir="${1}"
    if [[ -d "${rosella_clean_dir:?}/build" ]]; then rm -rf -- "${rosella_clean_dir:?}/build"; fi
}
empty=""
clean "${empty}"
printf '%s\n' "SHOULD NOT PRINT"
exit 0
