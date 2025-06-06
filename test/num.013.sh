#!/bin/bash

res=`echo '1e6' | $PUMP 'map num stdin'`
assert_eq "1000000" "$res"