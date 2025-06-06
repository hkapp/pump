#!/bin/bash

res=`echo 'nan' | $PUMP 'map num stdin'`
assert_eq "NaN" "$res"