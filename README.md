# Order Book

An attempt to build a simple [order book](https://www.investopedia.com/terms/o/order-book.asp) in [rust](https://rust-lang.org/)


## Copied information

* `config/order.csv` and some test cases are copied from various quantcup implementations


## Instructions to run dtrace 

* Remove old stacks file is any

    ```rm -f stacks.txt```

* Running dtrace to capture stack

    ```sudo dtrace -c './target/release/oms' -o stacks.txt -s profile.d```                                 

* Generating [flame graph](https://github.com/brendangregg/FlameGraph)

    ```$FLAME_GRAPH_HOME/stackcollapse.pl stacks.txt | $FLAME_GRAPH_HOME/flamegraph.pl > graph.svg```

    `$FLAME_GRAPH_HOME` is the path where you cloned/copied the repo
    
    
