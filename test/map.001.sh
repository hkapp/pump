#!/bin/bash

res=`echo "abc" | $PUMP 'map s/b/d/ stdin'`
assert_eq "$res" "adc"