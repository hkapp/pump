#!/bin/bash

res=`echo '-17.1' | $PUMP 'map num stdin'`
assert_eq "-17.1" "$res"