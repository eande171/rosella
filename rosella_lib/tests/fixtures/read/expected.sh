#!/bin/bash
printf '%s' "Name: "
IFS= read -r name
printf '%s\n' "Hello ${name}!"
exit 0
