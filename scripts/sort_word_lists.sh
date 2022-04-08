#!/usr/bin/env bash
for f in `ls src/word_lists/`
do
    sort -d -o "src/word_lists/$f" "src/word_lists/$f"
done
