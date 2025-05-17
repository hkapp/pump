#!/bin/bash

pump="$1"
res=`echo "abc" | $pump 'filter m/abc/ stdin'`
[ "$res" == "abc" ]
