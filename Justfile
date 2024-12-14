set dotenv-load := true
set unstable := true

# List all available commands
[private]
default:
    @just --list

[private]
test-cleanup:
    #!/usr/bin/env bash
    set -euo pipefail

    rm -rf test_bin
    echo "Test cleanup complete"

[private]
test-setup:
    #!/usr/bin/env bash
    set -euo pipefail

    cargo build

    TEST_DIR="test_bin"
    mkdir -p $TEST_DIR
    cd $TEST_DIR

    ln -sf ../target/debug/shadow .
    ln -sf shadow ls
    ln -sf shadow tree
    ln -sf shadow cat

    mkdir -p other_bin

    cd ..
    echo "Test setup complete"

clean:
    rm -rf target/

lint:
    @just --fmt
    cargo fmt
    cargo clippy

test:
    #!/usr/bin/env bash
    set -euo pipefail

    just test-setup

    cd test_bin
    export PATH="$PWD:$PATH"

    echo -e "\nTesting shadow add commands:"
    ./shadow add ls eza
    ./shadow add tree "eza --tree"
    ./shadow add cat bat --bin-path ./other_bin

    echo -e "\nTesting shadow list command:"
    ./shadow list

    echo -e "\nTesting shadowed commands:"
    ./ls --version
    ./tree --version
    ./cat --version

    echo -e "\nTesting shadow remove command:"
    ./shadow remove ls
    echo "After removing ls:"
    ./shadow list

    echo -e "\nTesting remove with specific bin path:"
    ./shadow remove cat --bin-path ./other_bin
    echo "After removing cat:"
    ./shadow list

    echo -e "\nTesting command with --raw flag:"
    if ! ./tree --raw --version; then
        echo "Raw command should execute original binary"
    fi

    echo -e "\nTesting non-existent shadow:"
    if ./shadow remove nonexistent; then
        echo "Error: remove should fail for non-existent shadow"
        exit 1
    else
        echo "Successfully detected non-existent shadow"
    fi

    echo -e "\nTesting invalid commands:"
    if ./shadow add; then
        echo "Error: add should fail without arguments"
        exit 1
    else
        echo "Successfully detected invalid add command"
    fi

    cd ..
    just test-cleanup

