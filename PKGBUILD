# Maintainer: Kien H. Luu <kien.luuhoang.arch@gmail.com>

pkgname=hyprvisor
pkgver=0.3.2
pkgrel=1
pkgdesc="Hyprvisor is a backend watcher for Hyprland."
arch=("x86_64")
url="https://github.com/lulkien/hyprvisor"
license=('UNLICENSE')
source=("$pkgname-v$pkgver-$arch-unknown-linux-gnu.tar.gz::$url/releases/download/v$pkgver/$pkgname-v$pkgver-$arch-unknown-linux-gnu.tar.gz")
sha256sums=('2e8086efe23c28fcab754e5c305827cb133adf27d8d8c3f08e03e8684daf663e')
provides=()
depends=()

package() {
    cd "$srcdir/$pkgname-v$pkgver-$arch-unknown-linux-gnu"
    install -Dm 755 hyprvisor "${pkgdir}/usr/bin/hyprvisor"
}
