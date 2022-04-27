# About

This library is part of [RIOT-rs](https://github.com/future-proof-iot/RIOT-rs).

This module provides a FIFO index queue that can be used for implementing
a ring buffer. It works in `no_std` settings.

It keeps track of indexes from 0..N (with N being a power of two).

`put()` marks an index "used".

`get()` returns an indexe that has been `put()` (if any) and marks it unused.

`peek()` returns the index that `get()` would return next (if any) without
marking it unused.

All operations are O(1).
