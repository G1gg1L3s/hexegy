# Hexegy

Hex encoded/decode data and print to standard output.

This is minimal CLI utility which I created only for myself, because I need a simple tool to convert to/from hex easily in command line, and I always forgot what `od` flags or any other utility should I use.

I don't plan to publish it on `crates.io` but if you are interested in using it or contributing to it please let me know :).


## Usage

### Encoding

Encode from the stdin:

```console
$ openssl rand 16 | hexegy`
```

Encode from files:

```console
$ hexegy a.txt b.txt c.txt`
```

Note that `-` file is stdin.

### Decoding

From the stdin:
```console
$ echo "44676402" | hexegy -d`
```

From a file:
```console
$ hexegy -d a.txt`
```

### Additional flags

Wrap lines after some number of bytes: `-w` or `--wrap`

```console
$ hexegy -w 16 < /dev/urandom | head`
```

Ignore whitespaces when decoding: `-i` or `--ignore-whitespaces`.
By default, only newlines `'\n'` are ignored.

```console
$ echo "4467 64" | hexegy -d -i
```
