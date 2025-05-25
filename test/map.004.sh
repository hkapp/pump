#!/bin/bash

res=`echo -e "abc\nbac" | $PUMP 'map s/ab/d/ stdin'`
expected=`echo -e "dc\nbac"`
assert_eq "$res" "$expected"