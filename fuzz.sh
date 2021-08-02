#!/bin/sh
while true; do
    radamsa fuzz-requests/get-index | nc localhost 8081 | head
    echo
    radamsa fuzz-requests/post-new | nc localhost 8081 | head
    echo
    radamsa fuzz-requests/get-edit | nc localhost 8081 | head
    echo
    radamsa fuzz-requests/post-edit | nc localhost 8081 | head
    echo
    radamsa fuzz-requests/post-complete | nc localhost 8081 | head
    echo
done
