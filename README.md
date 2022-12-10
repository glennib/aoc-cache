# aoc-cache

A way of caching your input from the great and popular [Advent of Code].

This is an attempt to reduce server load for the creator.

Downloads using [`ureq`][ureq], stores cache in temporary files using
[`scratch`][scratch].

Since we use [`scratch`][scratch], a `cargo clean` will remove the cache and cause new downloads for new runs.

## Example

```rust
use aoc_cache::get;
// my.cookie is a file containing the cookie string.
const MY_COOKIE: &str = include_str!("my.cookie");
let input: Result<String, aoc_cache::Error> = // Grabs from web if
    get(              // it's the first run
        "https://adventofcode.com/2022/day/1/input", MY_COOKIE);
let input: Result<String, aoc_cache::Error> = // Grabs from cache
    get(
        "https://adventofcode.com/2022/day/1/input", MY_COOKIE);
```

> **Warning** If you use source control for your AoC solutions, take care to not
> check in any files that contain your cookie into source control!
>
> Example `.gitignore`:
> ```
> **/target/
> my.cookie
> ```

[Advent of Code]: https://adventofcode.com/

[ureq]: https://docs.rs/ureq/

[scratch]: https://docs.rs/scratch/
