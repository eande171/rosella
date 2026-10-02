#!/bin/bash
# Fixture Runner

set -u
shell="${1:?usage: run_fixtures.sh bash|batch [--update]}"
update="${2:-}"
fixtures="$(cd "$(dirname "$0")/fixtures" && pwd)"
failed=0

case "$shell" in
    bash) extension=sh ;;
    batch) extension=bat ;;
    *) echo "unknown shell: $shell" >&2; exit 2 ;;
esac

for fixture in "$fixtures"/*/; do
    fixture="${fixture%/}"
    name="$(basename "$fixture")"
    work="$(mktemp -d)"
    cp "$fixture/expected.$extension" "$work/"

    # Isolated Run With Fixed Input
    if [ "$shell" = bash ]; then
        actual="$(cd "$work" && echo "Bob Smith" | bash expected.sh 2>/dev/null; echo "exit $?")"
    else
        actual="$(cd "$work" && echo "Bob Smith" | cmd //c "$(cygpath -w "$work/expected.bat")" 2>/dev/null | tr -d '\r'; echo "exit ${PIPESTATUS[1]}")"
    fi
    rm -rf "$work"

    if [ "$update" = --update ]; then
        printf '%s\n' "$actual" > "$fixture/expected.out"
    elif [ "$actual" != "$(cat "$fixture/expected.out")" ]; then
        echo "FAIL $name"
        diff <(cat "$fixture/expected.out") <(printf '%s\n' "$actual") | sed 's/^/    /'
        failed=1
    else
        echo "ok   $name"
    fi
done

exit "$failed"
