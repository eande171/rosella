#!/bin/bash
name="Name: "
printf '%s' "${name}"
IFS= read -r name
printf '%s\n' "Hello ${name}!"
exit 0
