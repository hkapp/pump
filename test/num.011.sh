#!/bin/bash

res=`echo 'nAn' | $PUMP 'map num stdin'`
assert_eq "NaN" "$res"