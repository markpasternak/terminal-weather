#!/usr/bin/env bash
set -euo pipefail

duplicates="$(
  git ls-files \
    | sort -f \
    | awk '
        BEGIN {
          found = 0
          prev_key = ""
          prev_path = ""
          in_group = 0
        }

        {
          key = tolower($0)
          if (key == prev_key) {
            if (!in_group) {
              found = 1
              print "case-collision: " key
              print prev_path
              in_group = 1
            }
            print $0
          } else {
            if (in_group) {
              print ""
            }
            prev_key = key
            prev_path = $0
            in_group = 0
          }
        }

        END {
          if (in_group) {
            print ""
          }
          exit(found ? 1 : 0)
        }
      '
)"

if [[ -z "$duplicates" ]]; then
  echo "No case-colliding tracked paths."
else
  printf '%s' "$duplicates"
fi
