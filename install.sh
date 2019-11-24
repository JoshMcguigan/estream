#!/usr/bin/env sh
# This is slightly modified version of
#   https://github.com/autozimu/LanguageClient-neovim/blob/next/install.sh
# Try install by
#   - download binary
#   - build with cargo

set -o nounset    # error when referencing undefined variable
set -o errexit    # exit when command fails

if [ "$#" -ne 1 ]
then
  echo "Must pass version to script, like './install.sh v0.1.0'"
  exit 1
fi

version=$1
name=estream

try_curl() {
    command -v curl > /dev/null && \
        curl --fail --location "$1" --output bin/$name
}

try_wget() {
    command -v wget > /dev/null && \
        wget --output-document=bin/$name "$1"
}

download() {
    echo "Downloading bin/${name} ${version}..."
    url=https://github.com/JoshMcguigan/estream/releases/download/$version/${1}
    if (try_curl "$url" || try_wget "$url"); then
        chmod a+x bin/$name
        return
    else
        try_build || echo "Prebuilt binary might not be ready yet. Please check minutes later."
    fi
}

try_build() {
    if command -v cargo > /dev/null; then
        echo "Trying build locally ${version} ..."
		cargo install --path=. --force
    else
        return 1
    fi
}

bin=bin/estream
if [ -f "$bin" ]; then
    installed_version=$($bin --version)
    case "${installed_version}" in 
		*${version}*) echo "Version is equal to ${version}, skipping install" ; exit 0 ;;
    esac
fi

arch=$(uname -sm)
case "${arch}" in
    "Linux x86_64") download linux-estream ;;
    "Darwin x86_64") download macos-estream ;;
    *) echo "No pre-built binary available for ${arch}."; try_build ;;
esac
