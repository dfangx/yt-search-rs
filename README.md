# yt-search-rs [![Build Status](https://travis-ci.com/dfangx/yt-search-rs.svg?branch=master)](https://travis-ci.com/dfangx/yt-search-rs)

A Youtube search CLI tool in Rust. Simply runs a search on a given search term
and outputs to stdout. This way, output can be piped or utilized in scripts.
This project is in its early stage, so fine tuning and additional CLI options
to come.

## Build 

Build using `cargo build --release`. You can find the resulting binary in
`./target/release/`. Copy the `yt-search` binary into your $PATH.

## Usage 

Run `yt-search --help` to see available options and arguments
