#!/bin/bash

#cargo run hello_world.cpp -- $* 2>log.txt
cargo run ./tests/test.txt -- $* 2>log.txt
