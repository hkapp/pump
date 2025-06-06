#!/bin/bash

res=`echo 'Nan' | $PUMP 'map num stdin'`
assert_eq "NaN" "$res"