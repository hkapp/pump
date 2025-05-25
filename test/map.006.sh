#!/bin/bash

res=`echo -e "abc\nbac" | $PUMP 'map s/a(b)/\1d\1/ stdin'`
expected=`echo -e "bdbc\nbac"`
assert_eq "$res" "$expected"