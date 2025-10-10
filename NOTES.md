

Followed Chapter 14.3 of the Rust Language reference to set up the repo.

https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html


Structure:
- We have a top level workspace.
- The parser will be a lib
- The viewer will be a bin


```
cargo new fpga_arch_viewer

cargo new fpga_arch_parser --lib
```
