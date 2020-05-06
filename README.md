flexihash-rs
============

![Unit Tests](https://github.com/shish/flexihash-rs/workflows/Unit%20Tests/badge.svg)

A rust port of https://github.com/pda/flexihash , aiming for 1:1 compatibility


Usage Example
-------------

```
use flexihash::Flexihash;

let fh = Flexihash::new();

// bulk add
fh.add_targets(vec!['cache-1', 'cache-2', 'cache-3']);

// simple lookup
fh.lookup('object-a');  // "cache-1"
fh.lookup('object-b');  // "cache-2"

// add and remove
fh.add_target('cache-4');
fh.remove_target('cache-1');

// lookup with next-best fallback (for redundant writes)
fh.lookup_list('object', 2)  // ["cache-2", "cache-4"]

// remove cache-2, expect object to hash to cache-4
fh.remove_target('cache-2')
fh.lookup('object')  // "cache-4"
```
