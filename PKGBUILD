pkgname=jack-autoconnect
pkgver=0.13.2
pkgrel=1
pkgdesc="A jack connector for connecting ports automatically"
url="https://github.com/shiro/jack-autoconnect"
arch=(x86_64)
license=(MIT)
depends=(gcc-libs)
makedepends=(cargo git)
source=("git+https://github.com/shiro/jack-autoconnect#branch=master")
sha512sums=('SKIP')

prepare() {
  cd $pkgname
  cargo fetch --locked
}

build() {
  cd $pkgname
  cargo build --release --frozen
}

check() {
  cd $pkgname
  cargo test --release --locked
}

package() {
  cd $pkgname
  install -Dt "$pkgdir/usr/bin" target/release/jack-autoconnect
  install -Dt "$pkgdir/usr/share/doc/$pkgname" -m644 README.md
}
