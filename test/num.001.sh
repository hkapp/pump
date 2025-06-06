#!/bin/bash

res=`echo '123' | $PUMP 'map num stdin'`
assert_eq "123" "$res"