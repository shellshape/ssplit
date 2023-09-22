# ssplit

An extremely simple CLI tool to split string contents.

## Install

You can either download the latest release builds form the [Releases page](https://github.com/shellshape/ssplit/releases) or you can install it using cargo install.

```
cargo install ssplit
```

## Usage

```
❯ ssplit --help
Simple CLI tool to split string contents

Usage: ssplit [OPTIONS] --split <SPLIT> [FILE]

Arguments:
  [FILE]  A file to be read as input; If not provided, StdIn is used as input

Options:
  -s, --split <SPLIT>          The string on which the input is split
  -d, --delimiter <DELIMITER>  The delimiter for the split elements [default: "\n"]
  -i, --idx <IDX>              Only select a given index, indices or index ranges (separated by ','); Ranges are defined in the form of {start}-{end} (.i.e. 3-7)
  -h, --help                   Print help
  -V, --version                Print version
```

By default, all splitted elements are printed to StdOut delimited by the given delimiter (new line by default).

If you pass the `-i` or `--idx` parameter, you can define a specific index, indices or index ranges to be returned. For example:

```
ssplit mydata.txt -s '/' -i '1,3,6-9'
```

The example above will split the input and only print elements at index 1, 3, 6, 7, 8 and 9.

## Performance

Because the tool reads the input in chunks and directly prints the split output to StdOut, the runtime should be linear to the size of the input and the memory consumption should be constant.

