#!/bin/bash

res=`echo -e "abc\nbac" | $PUMP 'map s/b/d/ stdin'`
expected=`echo -e "adc\ndac"`
assert_eq "$res" "$expected"