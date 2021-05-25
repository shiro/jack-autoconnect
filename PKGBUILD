# Maintainer: shiro <shiro@usagi.io>

pkgname=jack-autoconnect
pkgver=1.0.0
pkgrel=1
pkgdesc="Rule based jack connection management"
arch=('x86_64' 'i686')
license=('MIT')
depends=()
makedepends=(rustup)

build() {
	cd ..
  cargo build --release --locked --all-features --target-dir=target
}

check() {
	cd ..
  cargo test --release --locked --target-dir=target
}

package() {
	cd ..
  install -Dm 755 target/release/${pkgname} -t "${pkgdir}/usr/bin"
}
