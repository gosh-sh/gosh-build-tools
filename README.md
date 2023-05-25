# GOSH

[GOSH Documentation](https://docs.gosh.sh/)

## Installation

```
wget -O - https://raw.githubusercontent.com/gosh-sh/gosh-build-tools/dev/install.sh | bash -s
```

By default, script installs latest release to the default path `$HOME/.gosh/`, but you can customize it with env variables:

```bash
TAG=0.3.0 BINARY_PATH=/usr/local/bin ./install.sh
```
