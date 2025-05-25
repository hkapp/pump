#!/bin/bash

res=`echo -e "abc\nbac" | $PUMP 'map s/a(b)/d\1/ stdin'`
expected=`echo -e "dbc\nbac"`
assert_eq "$res" "$expected"