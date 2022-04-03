<h1 align=center>
toipe
</h1>

<p align=center>
A trusty terminal typing tester.
</p>

<p align=center>
<a href="https://crates.io/crates/toipe"><img alt="Crates.io badge" src="https://img.shields.io/crates/v/toipe"></a>
<a href="https://github.com/Samyak2/toipe/actions/workflows/lints-and-checks.yml"><img src="https://github.com/Samyak2/toipe/actions/workflows/lints-and-checks.yml/badge.svg" alt="Lints and checks badge" /></a>
<a href="https://docs.rs/toipe/latest/toipe/"><img alt="docs.rs badge" src="https://img.shields.io/docsrs/toipe"></a>
</p>


![A demo of toipe showing a recording of a terminal, running the typing test 3 times and then looping](https://raw.githubusercontent.com/Samyak2/toipe/main/images/toipe.gif)

# Usage

Note: toipe was only tested on Linux. If you find any problems, please [open an issue](https://github.com/Samyak2/toipe/issues).

## Install

### From GitHub

Go to the [latest release](https://github.com/Samyak2/toipe/releases/latest), scroll down to "Assets" and download the correct file for your platform (`.zip` in case of Windows and Mac OS, `.tar.gz` in case of Linux). Unzip the file and run the binary inside.

### From Cargo

Alternatively, if you have the `cargo` tool (part of the Rust toolchain) installed on your system, you can use:

```
cargo install toipe
```

## Run typing test

toipe looks best on a nice terminal (such as Alacritty) with color and style support.

If installed through GitHub, run the binary directly:
```
./binary-name-here
```

If installed through `cargo`, use:
```
toipe
```

## Show less or more text

To change the number of words shown in each test, use the `-n` flag (default: 30):

```
toipe -n 10
```

```
toipe -n 100
```

## Use a different word list

By default (in versions `>0.1.1`), a list of top 250 English words is used and random words are selected from it.

To use the OS provided word list instead (default in versions `<=0.1.1`), use:
```
toipe -w os
```
Note: the OS word list varies a lot from system to system and usually has more than 100,000 words. This can lead to difficult and esoteric words appearing in the test, reducing your typing speed.

You can provide your own word list too (TODO: requirements and assumptions of wordlist):
```
toipe -w /path/to/word/list
```

# License

MIT
