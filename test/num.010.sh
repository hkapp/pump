#!/bin/bash

res=`echo 'naN' | $PUMP 'map num stdin'`
assert_eq "NaN" "$res"