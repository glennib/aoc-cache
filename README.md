# aoc-cache

A way of caching your input from the great and popular [Advent of Code].

This is an attempt to reduce server load for the creator.

Downloads using [`reqwest`][reqwest], stores cache in temporary files using
[`scratch`][scratch].

Since we use [`scratch`][scratch], a `cargo clean` will remove the cache and cause new downloads for new runs.

## Example

```rust
use aoc_cache::get_input_from_web_or_cache;
// my.cookie is a file containing the cookie string.
const MY_COOKIE: &str = include_str!("my.cookie");
let input: Result<String, aoc_cache::Error> = // Grabs from web if it's the first run
    get_input_from_web_or_cache("https://adventofcode.com/2022/day/1/input", MY_COOKIE);
let input: Result<String, aoc_cache::Error> = // Grabs from cache
    get_input_from_web_or_cache("https://adventofcode.com/2022/day/1/input", MY_COOKIE);
```

[Advent of Code]: https://adventofcode.com/

[reqwest]: https://docs.rs/reqwest/

[scratch]: https://docs.rs/scratch/