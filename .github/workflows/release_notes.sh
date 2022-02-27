#!/bin/bash

changelog_path="$1"
version="$2"

awk -v version="$version" '/(^#+ \[)|(<!-- next-url -->)/ { if (p) { exit }; if ($2 == "["version"]") { p=1 } } p' "$changelog_path"
grep "\[$version\]: " "$changelog_path"
