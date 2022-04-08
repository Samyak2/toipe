#!/usr/bin/env bash
set -euo pipefail
for f in `ls src/word_lists/`
do
    echo ">>>>>>>> Checking file $f"
    sort -c -d "src/word_lists/$f"
    grep -iv "^[a-z]*$" "src/word_lists/$f" && echo "non-alphabetic characters present in one of the word lists." && exit 1
    echo "<<<<<<<<"
done

