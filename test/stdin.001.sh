#!/bin/bash

pump="$1"
out=`echo "abc" | $pump 'stdin'`

if [ $out == "abc" ]; then
    exit 0
else
    exit 1
fi