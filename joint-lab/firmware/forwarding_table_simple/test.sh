#!/bin/bash
while true; do
    ./makedata > data.in
    ./forwarding_table_simple <data.in >simple.out
    ./forwarding_table <data.in >bitmap.out
    if diff bitmap.out simple.out; then
        printf "AC\n"
    else
        printf "Wa\n"
        exit 0
    fi
done