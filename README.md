# Superv

A simple process supervisor written in Rust.

## Description

This project provides a simple way to launch services, having them
automatically restart when they exit.

## Installation

- Clone this repo
- Edit `superv.conf`
- Run `cargo run`

## License

This code is under the BSD-2-Clause license.  See the LICENSE file for the full
text.

## Bugs

Setting output files for `stdout` and `stderr` doesn't work yet.

There are still many features that should be implemented before this is useful
enough for a production system.
