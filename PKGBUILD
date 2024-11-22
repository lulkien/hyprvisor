# Maintainer: Kien H. Luu <kien.luuhoang.arch@gmail.com>

pkgname=hyprvisor
pkgver=0.4.0
pkgrel=1
pkgdesc="Hyprvisor is a backend watcher for Hyprland."
arch=("x86_64")
url="https://github.com/lulkien/hyprvisor"
license=('UNLICENSE')
source=("$pkgname-v$pkgver-$arch-unknown-linux-gnu.tar.gz::$url/releases/download/v$pkgver/$pkgname-v$pkgver-$arch-unknown-linux-gnu.tar.gz")
sha256sums=('d30200d990dd3f0b986a4c9969fddff0dd8e3b0ea7b971ddd0ecd904ea86d3ef')
provides=()
depends=()

package() {
    cd "$srcdir/$pkgname-v$pkgver-$arch-unknown-linux-gnu"
    install -Dm 755 hyprvisor "${pkgdir}/usr/bin/hyprvisor"
}
