## queue-rs

queue-rs is a library implemented using rust based on the **Implement Lock-Free Queues** paper algorithm.


### quick start

```rust
let q = Queue::new();
q.enq(1);
assert_eq!(q.deq(), Some(1));
assert_eq!(q.deq(), None);
```


### benchmark

Lock-Free Queue VS std::sync::Mutex\<std::collections::LinkedList\>

main.rs benchmark output:

|    Benchmark   | Total time spent | Average time spent |
| ------------- | ----- | ------- |
| queue_loop_n(100000) | **17.843828ms** | **178ns** |
| list_loop_n(100000)  | 23.066353ms | 230ns |
| queue_thread_n_m(2, 100000) | **64.018836ms** | **320ns** |
| list_thread_n_m(2, 100000)  | 74.660454ms | 373ns |
| queue_thread_n_m(4, 100000) | **149.736868ms** | **374ns** |
| list_thread_n_m(4, 100000)  | 189.6352ms | 474ns |
| queue_thread_n_m(8, 100000) | **544.476377ms** | **680ns** |
| list_thread_n_m(8, 100000)  | 980.688619ms | 1225ns |


### reference

[Implementing Lock-Free Queues (1994)](http://citeseerx.ist.psu.edu/viewdoc/summary?doi=10.1.1.53.8674)

