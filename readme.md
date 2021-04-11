# Hexegy

Hex encoded/decode data and print to standard output.

I've implemented this because I need tool to convert to/from hex easily, and I always forgot what flags of `od`.

## Usage

### Encoding

Encode from stdin:

`$ hexegy`

Encode from files:

`$ hexegy a.txt b.txt c.txt`

Note that `-` file is stdin.

### Decoding

From stdin:

`$ echo "44676402" | hexegy -d`

From file:
`$ hexegy -d a.txt`

### Additional flags

Wrap lines when encoding: `-w` or `--wrap`

`$ hexegy -w 16 < /dev/urandom | head `

Ignore whitespaces when decoding: `-i` or `--ignore-whitespaces`
By default only newlines ('\n') are ignored.

`$ echo "4467 64" | hexegy -d -i`
