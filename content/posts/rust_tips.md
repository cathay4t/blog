---
title: "Tips for Rust Coding"
date: 2021-11-15T13:53:35+08:00
---

## Input {:?} quickly in vim

Add this line into your vimrc:
```vimrc
autocmd FileType rust inoremap <leader>d {:?}
```

In vim __insert__ mode, if you try `\d`, vim will convert that into `{:?}`.

## Find out the type name of specific variable when debugging

Add this line to your code

```rust
let a: () = var;
```

You will got compile failure with type name of your variable.

## Speed up the compiling with `lld` linker

In Fedora/RHEL/CentOS, you may:
 * Add this to you `.bashrc`: `export RUSTFLAGS="-C link-arg=-fuse-ld=lld"`
 * Install `lld`: `sudo dnf install lld -y`
