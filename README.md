# assume-rolers
assume-rolers is a tool to use a temporary AWS credentials.
Currently, assume-rolers supports Linux and macOS only.

## CHANGELOG
TBA

## Installation

No pre-built binaries are available so far.
Please build this tool from the source.

```bash
$ cargo install assume-rolers
```

or

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

## Credentials
assume-rolers depends on rusoto's [DefaultCredentialsProvider](https://rusoto.github.io/rusoto/rusoto_core/struct.DefaultCredentialsProvider.html) backed by [ChainProvider](https://rusoto.github.io/rusoto/rusoto_credential/struct.ChainProvider.html). So assume-rolers will look credentials in this order.

> 1. Environment variables: AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY
> 2. credential_process command in the AWS config file, usually located at ~/.aws/config.
> 3. AWS credentials file. Usually located at ~/.aws/credentials.
> 4. IAM instance profile. Will only work if running on an EC2 instance with an instance profile/role.

quoted from Rusoto's document.
