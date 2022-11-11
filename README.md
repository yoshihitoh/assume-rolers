# assume-rolers
assume-rolers is a tool to use a temporary AWS credentials.
Currently, assume-rolers supports Linux and macOS only.

## CHANGELOG
TBA

## Installation

No pre-built binaries are available so far.
Please build this tool from the source.

```bash
$ git clone https://github.com/yoshihitoh/assume-rolers
$ cd assume-rolers
$ cargo build --release
```

If you're planning to use this tool frequently, please copy the binary to a directory included in PATH.
```bash
$ cp ./target/release/assume-rolers ~/.local/bin/
```

## How to use
### Interactive mode
You can select a profile on the terminal.
If the role you selected requires MFA, you can also set a token code on the terminal.

```bash
$ assume-rolers
```

### Specifying the profile
You can pass a profile as a command line argument.
If the role you selected requires MFA, you can set a token code via `-t` or `--token` flag.

```bash
$ assume-rolers <PROFILE_NAME> [-t <TOKEN>]
```
