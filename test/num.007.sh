#!/bin/bash

res=`echo 'NaN' | $PUMP 'map num stdin'`
assert_eq "NaN" "$res"