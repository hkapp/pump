test_script="$1"
PUMP="$2"

# Note: exports will be local to this script run
export PUMP

# Define utility functions used by the individual tests
invalid_program () {
    cmdout=`echo "fail" | "$PUMP" "$1" 2>&1`
    cmdsta=$?
    echo "$cmdout"

    # Did the invocation actually fail?
    if [ $cmdsta -ne 0 ]; then
        # Did we get a panic?
        if echo "$cmdout" | grep -q "^thread.*panicked" ; then
            # We did panic
            exit 1
        else
            # No panic, all good
            exit 0
        fi
    else
        exit 1
    fi
}
export -f invalid_program

assert_eq () {
    left="$1"
    right="$2"
    echo "$left"
    [ "$left" == "$right" ]
}
export -f assert_eq

# TODO remove the second argument completely
bash "$test_script" "$PUMP"