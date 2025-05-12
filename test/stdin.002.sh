#!/bin/bash

pump="$1"
! echo "fail" | $pump 'stdin stdin'
