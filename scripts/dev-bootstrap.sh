#!/usr/bin/env bash
set -eu

__mise_bootstrap() {
  local script_dir project_dir cache_home
    script_dir=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
    project_dir=$( cd -- "$( dirname -- "$script_dir" )" &> /dev/null && pwd )
    export MISE_BOOTSTRAP_PROJECT_DIR="$project_dir"
    cache_home="${XDG_CACHE_HOME:-$HOME/.cache}/mise"
    export MISE_INSTALL_PATH="$cache_home/mise-2025.5.16"
    install() {
        #!/bin/sh
        set -eu

        #region logging setup
        if [ "${MISE_DEBUG-}" = "true" ] || [ "${MISE_DEBUG-}" = "1" ]; then
          debug() {
            echo "$@" >&2
          }
        else
          debug() {
            :
          }
        fi

        if [ "${MISE_QUIET-}" = "1" ] || [ "${MISE_QUIET-}" = "true" ]; then
          info() {
            :
          }
        else
          info() {
            echo "$@" >&2
          }
        fi

        error() {
          echo "$@" >&2
          exit 1
        }
        #endregion

        #region environment setup
        get_os() {
          os="$(uname -s)"
          if [ "$os" = Darwin ]; then
            echo "macos"
          elif [ "$os" = Linux ]; then
            echo "linux"
          else
            error "unsupported OS: $os"
          fi
        }

        get_arch() {
          musl=""
          if type ldd >/dev/null 2>/dev/null; then
            libc=$(ldd /bin/ls | grep 'musl' | head -1 | cut -d ' ' -f1)
            if [ -n "$libc" ]; then
              musl="-musl"
            fi
          fi
          arch="$(uname -m)"
          if [ "$arch" = x86_64 ]; then
            echo "x64$musl"
          elif [ "$arch" = aarch64 ] || [ "$arch" = arm64 ]; then
            echo "arm64$musl"
          elif [ "$arch" = armv7l ]; then
            echo "armv7$musl"
          else
            error "unsupported architecture: $arch"
          fi
        }

        get_ext() {
          if [ -n "${MISE_INSTALL_EXT:-}" ]; then
            echo "$MISE_INSTALL_EXT"
          elif [ -n "${MISE_VERSION:-}" ] && echo "$MISE_VERSION" | grep -q '^v2024'; then
            # 2024 versions don't have zstd tarballs
            echo "tar.gz"
          elif tar_supports_zstd; then
            echo "tar.zst"
          elif command -v zstd >/dev/null 2>&1; then
            echo "tar.zst"
          else
            echo "tar.gz"
          fi
        }

        tar_supports_zstd() {
          # tar is bsdtar or version is >= 1.31
          if tar --version | grep -q 'bsdtar' && command -v zstd >/dev/null 2>&1; then
            true
          elif tar --version | grep -q '1\.(3[1-9]|[4-9][0-9]'; then
            true
          else
            false
          fi
        }

        shasum_bin() {
          if command -v shasum >/dev/null 2>&1; then
            echo "shasum"
          elif command -v sha256sum >/dev/null 2>&1; then
            echo "sha256sum"
          else
            error "mise install requires shasum or sha256sum but neither is installed. Aborting."
          fi
        }

        get_checksum() {
          version=$1
          os="$(get_os)"
          arch="$(get_arch)"
          ext="$(get_ext)"
          url="https://github.com/jdx/mise/releases/download/v${version}/SHASUMS256.txt"

          # For current version use static checksum otherwise
          # use checksum from releases
          if [ "$version" = "v2025.5.16" ]; then
            checksum_linux_x86_64="308383c996e356104a10241dad26292c9e0817b6f88548dcf14e851269e67128  ./mise-v2025.5.16-linux-x64.tar.gz"
            checksum_linux_x86_64_musl="68880df804c2e9d7a66754330463f0809a8cc184745949d2ccf4a64162eacb61  ./mise-v2025.5.16-linux-x64-musl.tar.gz"
            checksum_linux_arm64="861ab2f80f4b01308b35254e309adef272c32dd47f329f67da9babf5a79f9c28  ./mise-v2025.5.16-linux-arm64.tar.gz"
            checksum_linux_arm64_musl="2e6abaa0dfa03f3f079a856780d7cfd60d7e68d7bb032d52d34176f9ef2f038b  ./mise-v2025.5.16-linux-arm64-musl.tar.gz"
            checksum_linux_armv7="4d8ce70960a1e68455eb891696dc4c949e391f1bd15cdd2cd9cdfe1aa0efb990  ./mise-v2025.5.16-linux-armv7.tar.gz"
            checksum_linux_armv7_musl="42dec96ecdf844a2e89cb7d7e7b4582666aff5848e01565796a4386f1ce9cdaa  ./mise-v2025.5.16-linux-armv7-musl.tar.gz"
            checksum_macos_x86_64="e0ae10a5ba9f2e0ee5ee2c7cea9b7064e37cff562f9e06f259e431460e4d928e  ./mise-v2025.5.16-macos-x64.tar.gz"
            checksum_macos_arm64="dff5b38e4e7c7c7e95ebfae77ef216c0ff5cc0a2232ee3bc48286975c836f5cf  ./mise-v2025.5.16-macos-arm64.tar.gz"
            checksum_linux_x86_64_zstd="180bad59014d612e53dd9fe3e371ab63a86d93a5ea630ea19418e61f90556b96  ./mise-v2025.5.16-linux-x64.tar.zst"
            checksum_linux_x86_64_musl_zstd="45361fdbb9b1a88fb8f319f99a0db6b06417f98d09fb1481eb9244842afb517f  ./mise-v2025.5.16-linux-x64-musl.tar.zst"
            checksum_linux_arm64_zstd="b62f81af22f46c946db8d03acdacf7f20751ca6719309884ddcc77e1b50f4151  ./mise-v2025.5.16-linux-arm64.tar.zst"
            checksum_linux_arm64_musl_zstd="73e85d43125b42b1d95cb36a4e2e02b4704dc65cb9b2dea083667ff4ee1e8ecc  ./mise-v2025.5.16-linux-arm64-musl.tar.zst"
            checksum_linux_armv7_zstd="dfcd6792e9dce0e8427941d738bcffd3b376b8f9d553df6ddfc57202eebbe250  ./mise-v2025.5.16-linux-armv7.tar.zst"
            checksum_linux_armv7_musl_zstd="4b5307803b936340190ba9321e625313fc7008058eef915c296d8cd9510cc7a0  ./mise-v2025.5.16-linux-armv7-musl.tar.zst"
            checksum_macos_x86_64_zstd="e77b6d8c433a248d53dde333c411330bdc1715b1e61f54596542ac9733794873  ./mise-v2025.5.16-macos-x64.tar.zst"
            checksum_macos_arm64_zstd="65d7bcf3e308e675798650e556a400a029e8c9c12e2aa053557936f2f3611d04  ./mise-v2025.5.16-macos-arm64.tar.zst"

            # TODO: refactor this, it's a bit messy
            if [ "$(get_ext)" = "tar.zst" ]; then
              if [ "$os" = "linux" ]; then
                if [ "$arch" = "x64" ]; then
                  echo "$checksum_linux_x86_64_zstd"
                elif [ "$arch" = "x64-musl" ]; then
                  echo "$checksum_linux_x86_64_musl_zstd"
                elif [ "$arch" = "arm64" ]; then
                  echo "$checksum_linux_arm64_zstd"
                elif [ "$arch" = "arm64-musl" ]; then
                  echo "$checksum_linux_arm64_musl_zstd"
                elif [ "$arch" = "armv7" ]; then
                  echo "$checksum_linux_armv7_zstd"
                elif [ "$arch" = "armv7-musl" ]; then
                  echo "$checksum_linux_armv7_musl_zstd"
                else
                  warn "no checksum for $os-$arch"
                fi
              elif [ "$os" = "macos" ]; then
                if [ "$arch" = "x64" ]; then
                  echo "$checksum_macos_x86_64_zstd"
                elif [ "$arch" = "arm64" ]; then
                  echo "$checksum_macos_arm64_zstd"
                else
                  warn "no checksum for $os-$arch"
                fi
              else
                warn "no checksum for $os-$arch"
              fi
            else
              if [ "$os" = "linux" ]; then
                if [ "$arch" = "x64" ]; then
                  echo "$checksum_linux_x86_64"
                elif [ "$arch" = "x64-musl" ]; then
                  echo "$checksum_linux_x86_64_musl"
                elif [ "$arch" = "arm64" ]; then
                  echo "$checksum_linux_arm64"
                elif [ "$arch" = "arm64-musl" ]; then
                  echo "$checksum_linux_arm64_musl"
                elif [ "$arch" = "armv7" ]; then
                  echo "$checksum_linux_armv7"
                elif [ "$arch" = "armv7-musl" ]; then
                  echo "$checksum_linux_armv7_musl"
                else
                  warn "no checksum for $os-$arch"
                fi
              elif [ "$os" = "macos" ]; then
                if [ "$arch" = "x64" ]; then
                  echo "$checksum_macos_x86_64"
                elif [ "$arch" = "arm64" ]; then
                  echo "$checksum_macos_arm64"
                else
                  warn "no checksum for $os-$arch"
                fi
              else
                warn "no checksum for $os-$arch"
              fi
            fi
          else
            if command -v curl >/dev/null 2>&1; then
              debug ">" curl -fsSL "$url"
              checksums="$(curl --compressed -fsSL "$url")"
            else
              if command -v wget >/dev/null 2>&1; then
                debug ">" wget -qO - "$url"
                stderr=$(mktemp)
                checksums="$(wget -qO - "$url")"
              else
                error "mise standalone install specific version requires curl or wget but neither is installed. Aborting."
              fi
            fi
            # TODO: verify with minisign or gpg if available

            checksum="$(echo "$checksums" | grep "$os-$arch.$ext")"
            if ! echo "$checksum" | grep -Eq "^([0-9a-f]{32}|[0-9a-f]{64})"; then
              warn "no checksum for mise $version and $os-$arch"
            else
              echo "$checksum"
            fi
          fi
        }

        #endregion

        download_file() {
          url="$1"
          filename="$(basename "$url")"
          cache_dir="$(mktemp -d)"
          file="$cache_dir/$filename"

          info "mise: installing mise..."

          if command -v curl >/dev/null 2>&1; then
            debug ">" curl -#fLo "$file" "$url"
            curl -#fLo "$file" "$url"
          else
            if command -v wget >/dev/null 2>&1; then
              debug ">" wget -qO "$file" "$url"
              stderr=$(mktemp)
              wget -O "$file" "$url" >"$stderr" 2>&1 || error "wget failed: $(cat "$stderr")"
            else
              error "mise standalone install requires curl or wget but neither is installed. Aborting."
            fi
          fi

          echo "$file"
        }

        install_mise() {
          version="${MISE_VERSION:-v2025.5.16}"
          version="${version#v}"
          os="$(get_os)"
          arch="$(get_arch)"
          ext="$(get_ext)"
          install_path="${MISE_INSTALL_PATH:-$HOME/.local/bin/mise}"
          install_dir="$(dirname "$install_path")"
          tarball_url="https://github.com/jdx/mise/releases/download/v${version}/mise-v${version}-${os}-${arch}.${ext}"

          cache_file=$(download_file "$tarball_url")
          debug "mise-setup: tarball=$cache_file"

          debug "validating checksum"
          cd "$(dirname "$cache_file")" && get_checksum "$version" | "$(shasum_bin)" -c >/dev/null

          # extract tarball
          mkdir -p "$install_dir"
          rm -rf "$install_path"
          cd "$(mktemp -d)"
          if [ "$(get_ext)" = "tar.zst" ] && ! tar_supports_zstd; then
            zstd -d -c "$cache_file" | tar -xf -
          else
            tar -xf "$cache_file"
          fi
          mv mise/bin/mise "$install_path"
          info "mise: installed successfully to $install_path"
        }

        after_finish_help() {
          case "${SHELL:-}" in
          */zsh)
            info "mise: run the following to activate mise in your shell:"
            info "echo \"eval \\\"\\\$($install_path activate zsh)\\\"\" >> \"${ZDOTDIR-$HOME}/.zshrc\""
            info ""
            info "mise: run \`mise doctor\` to verify this is setup correctly"
            ;;
          */bash)
            info "mise: run the following to activate mise in your shell:"
            info "echo \"eval \\\"\\\$($install_path activate bash)\\\"\" >> ~/.bashrc"
            info ""
            info "mise: run \`mise doctor\` to verify this is setup correctly"
            ;;
          */fish)
            info "mise: run the following to activate mise in your shell:"
            info "echo \"$install_path activate fish | source\" >> ~/.config/fish/config.fish"
            info ""
            info "mise: run \`mise doctor\` to verify this is setup correctly"
            ;;
          *)
            info "mise: run \`$install_path --help\` to get started"
            ;;
          esac
        }

        install_mise
        if [ "${MISE_INSTALL_HELP-}" != 0 ]; then
          after_finish_help
        fi

        cd "$MISE_BOOTSTRAP_PROJECT_DIR"
    }
    local MISE_INSTALL_HELP=0
    test -f "$MISE_INSTALL_PATH" || install
}
__mise_bootstrap
exec "$MISE_INSTALL_PATH" "$@"
