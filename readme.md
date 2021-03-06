# Hexegy

Hex encoded/decode data and print to standard output.

This is minimal CLI utility which I created only for myself, because I need a simple tool to convert to/from hex easily in command line, and I always forgot what `od` flags or any other utility I should use.

I don't plan to publish it on `crates.io`, but if you are interested in using it or contributing to it please let me know :).

---
## Usage

### Encoding

Encode from the stdin:

```console
$ openssl rand 16 | hexegy
```

Encode from files:

```console
$ hexegy -f a.txt b.txt c.txt
```

Note that `-` file is stdin.

Encode from argument:

```console
$ hexegy "in rust we trust" # will produce "7265777269746520696e2072757374"
```

### Decoding

From the stdin:
```console
$ echo "44676402" | hexegy -d
```

From a file:
```console
$ hexegy -d -f a.txt
```

From a string arg:

```console
$ hexegy -d 77686f2077696c6c2065766572207265616420746869733f3f3f
```

### Additional flags

Wrap lines after some number of bytes: `-w` or `--wrap`

```console
$ hexegy -w 16 < /dev/urandom | head
```

Ignore whitespaces when decoding: `-i` or `--ignore-whitespaces`.
By default, only newlines `'\n'` are ignored.

```console
$ echo "4467 64" | hexegy -d -i
```

You can also specify the prefix, that will be printed before each byte.
Use `-p` or `--prefix` option:

```console
$ echo "hexegy" | hexegy -p '\x' # will produce "\x68\x65\x78\x65\x67\x79\x0a"
```
