#!/usr/bin/env bash


cargo build --release

# Change this to suit you
FLAME_GRAPH_HOME=../../FlameGraph

rm -f stacks.txt

sudo dtrace -c './target/release/oms' -o stacks.txt -s profile.d

${FLAME_GRAPH_HOME}/stackcollapse.pl stacks.txt | ${FLAME_GRAPH_HOME}/flamegraph.pl > graph.svg
