#!/bin/bash

# Run all experimental add tests
for i in {1..6}; do
    echo "Running add test $i"
    # hide all stderr and stdout output from the test, but print the exit status
    CC=cc cargo run example add $i > /dev/null 2>&1
    status=$?
    echo "Test $i exited with status $status"
done