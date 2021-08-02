#!/bin/sh
while true; do
    radamsa thruster-fuzz-requests/get-index | nc localhost 8082 | head
    echo
    radamsa thruster-fuzz-requests/post-new | nc localhost 8082 | head
    echo
    radamsa thruster-fuzz-requests/get-edit | nc localhost 8082 | head
    echo
    radamsa thruster-fuzz-requests/post-edit | nc localhost 8082 | head
    echo
    radamsa thruster-fuzz-requests/post-complete | nc localhost 8082 | head
    echo
done
