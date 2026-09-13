Instructions use Ubuntu package names. Package names may differ on other distros.

If it's inconvenient to install the latest version of [just](https://github.com/casey/just), use the without just instructions. The catch is that the without just instructions are more likely to change in the future, so if you're packaging openscq30 and the latest version of just is easily available, prefer the with just instructions.

## Building openscq30-cli on Linux

1. Install the latest version of rust

### Without just

2. Run `cargo build --package openscq30-cli --profile release-fast` (or `cargo build --package openscq30-cli --release`, but it's very slow to build)
3. The compiled binary can be found at `target/release-fast/openscq30`

### With just

2. Run `just build-cli-fast` (or `just build-cli` but it's very slow to build)
3. The compiled binary can be found at `build-output/openscq30`

## Building openscq30-gui on Linux

1. Install the latest version of rust
2. Install pkg-config libdbus-1-dev libxkbcommon-dev

### Without just

3. Run `cargo build --package openscq30-gui --profile release-fast` (or `cargo build --package openscq30-gui --release`, but it's very slow to build)
4. The compiled binary can be found at `target/release-fast/openscq30-gui`

### With just

3. Run `just build-gui-fast` (or `just build-gui` but it's very slow to build)
4. The compiled binary can be found at `build-output/openscq30-gui`

## Runtime Dependencies

- [cosmic-icons](https://github.com/pop-os/cosmic-icons/): if a package isn't available, clone the git repo and run `just install`.

## Building on postmarketOS

postmarketOS is based on Alpine Linux and uses `abuild` for packaging. The following instructions assume you are building inside the postmarketOS build environment (e.g. via `pmbootstrap` chroot or on device).

### Dependencies

```sh
apk add rust cargo pkgconfig dbus-dev libxkbcommon-dev just
```

### Build

1. Clone the repository and enter the project directory.
2. Run `just build-gui-fast` (or `just build-gui` for a fully optimized build).
3. The compiled binary is at `build-output/openscq30-gui`.

### Packaging with abuild (optional)

To create an Alpine/package-compatible package, add a pmaport under `pmaports/main/openscq30/` with the following minimal `APKBUILD`:

```sh
pkgname=openscq30
_pkgname=OpenSCQ30
pkgver=2.8.0
pkgrel=0
pkgdesc="GUI for Soundcore headphones and earbuds"
url="https://github.com/Oppzippy/OpenSCQ30"
arch="all"
license="GPL-3.0-or-later"
makedepends="just rust cargo pkgconfig dbus-dev libxkbcommon-dev"
depends=""
source="$pkgname-$pkgver.tar.gz::https://github.com/Oppzippy/$_pkgname/archive/refs/tags/v$pkgver.tar.gz"
build() {
    just build-gui-fast
}
package() {
    install -Dm755 build-output/openscq30-gui "$pkgdir/usr/bin/openscq30-gui"
}
```

Then run `abuild-rsync` or `abuild -r` from the pmaports directory to build.
