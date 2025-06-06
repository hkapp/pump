#!/bin/bash

res=`echo '1.23' | $PUMP 'map num stdin'`
assert_eq "1.23" "$res"