#!/bin/bash

pump="$1"
out=`echo "abc" | $pump 'stdin'`

[ "$out" == "abc" ]