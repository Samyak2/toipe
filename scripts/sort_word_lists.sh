#!/usr/bin/env bash
for f in `ls src/word_lists/`
do
    LC_COLLATE=en_US.UTF-8 sort -d -o "src/word_lists/$f" "src/word_lists/$f"
done
