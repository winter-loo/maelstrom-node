#!/bin/bash
# Download malestrom from https://github.com/jepsen-io/maelstrom/releases/download/v0.2.3/maelstrom.tar.bz2
# and put it in the same directory as this script.
# then test maelstrom-node for each chanllge.
# Example:
# ./test.sh c1
# ./test.sh c2
# ./test.sh c3a
# ./test.sh c3b

# define the test commands for each challenge with the first argument as the
# key and the second argument as the value
command_prefix="maelstrom/maelstrom test --bin target/debug/maelstrom-node"
c1_command="$command_prefix -w echo --node-count 1 --time-limit 10"
c2_command="$command_prefix -w unique-ids --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition"
c3a_command="$command_prefix -w broadcast --node-count 1 --time-limit 20 --rate 10"
c3b_command="$command_prefix -w broadcast --node-count 5 --time-limit 20 --rate 10"
c3c_command="$command_prefix -w broadcast --node-count 5 --time-limit 20 --rate 10 --nemesis partition"
c3d_command="$command_prefix -w broadcast --node-count 25 --time-limit 20 --rate 100 --latency 100"
c3e_command="$command_prefix -w broadcast --node-count 25 --time-limit 20 --rate 100 --latency 100"
c4_command="$command_prefix -w g-counter --node-count 3 --rate 100 --time-limit 20 --nemesis partition"
c5a_command="$command_prefix -w kafka --node-count 1 --concurrency 2n --time-limit 20 --rate 1000"
c5b_command="$command_prefix -w kafka --node-count 2 --concurrency 2n --time-limit 20 --rate 1000"
c5c_command="$command_prefix -w kafka --node-count 1 --concurrency 2n --time-limit 20 --rate 1000"
c6a_command="$command_prefix -w txn-rw-register --node-count 1 --time-limit 20 --rate 1000 --concurrency 2n --consistency-models read-uncommitted --availability total"
c6b_command="$command_prefix -w txn-rw-register --node-count 2 --concurrency 2n --time-limit 20 --rate 1000 --consistency-models read-uncommitted --availability total --nemesis partition"
c6c_command="$command_prefix -w txn-rw-register --node-count 2 --concurrency 2n --time-limit 20 --rate 1000 --consistency-models read-committed --availability total –-nemesis partition"

command_prefix="maelstrom/maelstrom test --bin target/debug/maelstrom-txn"
c7a_command="$command_prefix -w txn-list-append --node-count 1 --time-limit 10"
c7b_command="$command_prefix -w txn-list-append --node-count 2 --time-limit 10"

if [ "$(uname)" == "Darwin" ]; then
    # use arrays for macos because bash doesn't support associative arrays
    tests=(
        "c1:$c1_command"
        "c2:$c2_command"
        "c3a:$c3a_command"
        "c3b:$c3b_command"
        "c3c:$c3c_command"
        "c3d:$c3d_command"
        "c3e:$c3e_command"
        "c4:$c4_command"
        "c5a:$c5a_command"
        "c5b:$c5b_command"
        "c5c:$c5c_command"
        "c6a:$c6a_command"
        "c6b:$c6b_command"
        "c6c:$c6c_command"
        "c7a:$c7a_command"
        "c7b:$c7b_command"
    )
else
    declare -A tests
    tests["c1"]="$c1_command"
    tests["c2"]="$c2_command"
    tests["c3a"]="$c3a_command"
    tests["c3b"]="$c3b_command"
    tests["c3c"]="$c3c_command"
    tests["c3d"]="$c3d_command"
    tests["c3e"]="$c3e_command"
    tests["c4"]="$c4_command"
    tests["c5a"]="$c5a_command"
    tests["c5b"]="$c5b_command"
    tests["c5c"]="$c5c_command"
    tests["c6a"]="$c6a_command"
    tests["c6b"]="$c6b_command"
    tests["c6c"]="$c6c_command"
    tests["c7a"]="$c7a_command"
    tests["c7b"]="$c7b_command"
fi

if [ "$#" -eq 0 ]; then
    echo "Usage: $0 {<challenge> | serve}"
    echo "the challenges are:"
    if [ "$(uname)" == "Darwin" ]; then
        for key in "${tests[@]}"; do
            echo "${key%%:*}"
        done
    else
        for key in "${!tests[@]}"; do
            echo "$key"
        done
    fi
    exit 1
fi

# extract maelstrom if it doesn't exist
if [ ! -d maelstrom ]; then
    [ -f maelstrom.tar.bz2 ] && rm maelstrom.tar.bz2
    wget https://github.com/jepsen-io/maelstrom/releases/download/v0.2.3/maelstrom.tar.bz2
    if [ $? -ne 0 ]; then
        echo "ERROR: Can't download maelstrom!"
        exit 1
    fi
    tar xvf maelstrom.tar.bz2
    if [ $? -ne 0 ]; then
        echo "ERROR: Can't extract maelstrom!"
        exit 1
    fi
    rm maelstrom.tar.bz2
fi

# if [ "$1" = "c7b" ]
# then
#   cargo build --bin maelstrom-txn --features lin_kv
# else
#   cargo build
# fi

if [ "$1" == "serve" ]; then
    maelstrom/maelstrom serve
    exit 0
fi

echo "Testing $1"
if [ "$(uname)" == "Darwin" ]; then
    for key in "${tests[@]}"; do
        if [ "${key%%:*}" == "$1" ]; then
            eval "${key##*:}"
            exit 0
        fi
    done
else
    ${tests[$1]}
fi
