#!/bin/bash

res=`echo -e "abc\nbac" | $PUMP 'map s/[ab]/d/ stdin'`
expected=`echo -e "dbc\ndac"`
assert_eq "$res" "$expected"