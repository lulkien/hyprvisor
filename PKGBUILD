# Maintainer: Kien H. Luu <kien.luuhoang.arch@gmail.com>

pkgname=hyprvisor
pkgver=0.4.1
pkgrel=1
pkgdesc="Hyprvisor is a backend watcher for Hyprland."
arch=("x86_64")
url="https://github.com/lulkien/hyprvisor"
license=('UNLICENSE')
source=("$pkgname-v$pkgver-$arch-unknown-linux-gnu.tar.gz::$url/releases/download/v$pkgver/$pkgname-v$pkgver-$arch-unknown-linux-gnu.tar.gz")
sha256sums=('f165be378c285357f95cbf00af9d7f003b2c790f62d194e8073d77b3fb709c66')
provides=()
depends=()

package() {
    cd "$srcdir/$pkgname-v$pkgver-$arch-unknown-linux-gnu"
    install -Dm 755 hyprvisor "${pkgdir}/usr/bin/hyprvisor"
}
