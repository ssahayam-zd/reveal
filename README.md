# Reveal

Converts scala class files into the matching scala source files by running them through `scalap`.

## Usage

Run:

```
reveal -h
```

which results in the following options:

```
Converts Scala class files into the matching Scala source files

Usage: reveal --classes-dir <CLASSES_DIR> --output-dir <OUTPUT_DIR>

Options:
  -c, --classes-dir <CLASSES_DIR>  The directory with the Scala class files
  -o, --output-dir <OUTPUT_DIR>    The directory that will contain the generated Scala source files
  -h, --help                       Print help information
  -V, --version                    Print version information
```

## Building

```
cargo build --release
```

The executable can be found at: `target/release/reveal`