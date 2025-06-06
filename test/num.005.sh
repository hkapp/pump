#!/bin/bash

res=`echo '23.' | $PUMP 'map num stdin'`
assert_eq "23" "$res"