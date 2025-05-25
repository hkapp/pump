#!/bin/bash

res=`echo -e "abc\nbac" | $PUMP 'map m/ab/ stdin'`
expected=`echo -e "true\nfalse"`
assert_eq "$res" "$expected"