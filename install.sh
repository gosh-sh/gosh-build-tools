#!/bin/bash

set -e

REPO_OWNER=gosh-sh
REPO=gosh-build-tools
if [[ -z "${TAG}" ]]; then
  echo ""
  echo "Downloading latest release of gosh"
  echo ""
  TAG=latest
else
  echo ""
  echo "Downloading gosh tag: $TAG"
  echo ""
  TAG="tags/$TAG"
fi

# Check OS and architecture
if [[ "$OSTYPE" == "linux-gnu" ]]; then
    if [[ $(uname -m) == "x86_64" ]]; then
        TAR="gosh-linux-amd64.tar.gz"
    else
        TAR="gosh-linux-arm64.tar.gz"
    fi
else
    echo "Only \"Linux\" is supported - not \"$OSTYPE\""
    exit 1
fi

OLD_TAR="${TAR%.*}"
TEMP_DIR="gosh_tmp"
[ -f $TAR ] && rm $TAR
[ -d $OLD_TAR ] && rm -r $OLD_TAR
[ -d $TEMP_DIR ] && rm -r $TEMP_DIR

GH_API="https://api.github.com"
GH_REPO="$GH_API/repos/${REPO_OWNER}/${REPO}"
GH_TAGS="$GH_REPO/releases/$TAG"

# Read asset tags.
response=$(curl -s "$GH_TAGS")

# Get ID of the asset based on the given name.
eval $(echo "$response" | grep -C3 "name.:.\+$TAR" | grep -w id | tr : = | tr -cd '[[:alnum:]]=')
[ "$id" ] || {
  OLD=TRUE
  eval $(echo "$response" | grep -C3 "name.:.\+$OLD_TAR" | grep -w id | tr : = | tr -cd '[[:alnum:]]=')
  [ "$id" ] || {
    echo "Error: Failed to get asset id, response: $response" | awk 'length($0)<100' >&2
    exit 1
  }
}

wget --content-disposition --no-cookie -q --header "Accept: application/octet-stream" "$GH_REPO/releases/assets/$id" --show-progress

# Create the temporary directory
mkdir $TEMP_DIR

# Unpack the downloaded tar archive to the temporary directory
if [[ -z "${OLD}" ]]; then
  tar -xvzf $TAR -C $TEMP_DIR
  rm -f $TAR
else
  tar -xf $OLD_TAR -C $TEMP_DIR
  rm -f $OLD_TAR
fi

DEFAULT_PATH=$HOME/.gosh/
BINARY_PATH="${BINARY_PATH:-$DEFAULT_PATH}"

# Move the contents from the temporary directory to the desired installation path
mv $TEMP_DIR/gosh $BINARY_PATH

echo ""
echo "Binary was installed to $BINARY_PATH"
echo ""

# Check if the binary path is added to .bashrc
ALREADY_ADDED=$(cat "$HOME"/.bashrc | grep "export PATH=\$PATH:\$HOME/.gosh" | wc -l)
if [ $ALREADY_ADDED -lt 1 ]; then
  echo "export PATH=\$PATH:\$HOME/.gosh" >>"$HOME"/.bashrc
  export PATH=$PATH:\$HOME/.gosh
fi

# Remove the temporary directory
rm -r $TEMP_DIR