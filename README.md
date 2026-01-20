# zed-mono-debugger

The zed mono debugger is mainly for unity debugging.

Since [Dotrush](https://github.com/JaneySprings/DotRush) already implemented a DAP agent for `monodbg`, I decided to use it instead of implementing another in rust.

> Note: The Dotrush version of DAP agent is download from open vsx since the official vsx market download api closed.

## Installation

Download the git repository and unzip it, then install develop extension in zed with root directory selected.

## .zed/debug.json

Debug config file should be:

```.zed/debug.json JSON
[
  {
    "label": "unity",
    "request": "attach",
    "adapter": "monodbg",
  },
]
```
