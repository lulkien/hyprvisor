# Maintainer: Kien H. Luu <kien.luuhoang.arch@gmail.com>

pkgname=hyprvisor
pkgver=0.4.2
pkgrel=1
pkgdesc="Hyprvisor is a backend watcher for Hyprland."
arch=("x86_64")
url="https://github.com/lulkien/hyprvisor"
license=('UNLICENSE')
source=("$pkgname-v$pkgver-$arch-unknown-linux-gnu.tar.gz::$url/releases/download/v$pkgver/$pkgname-v$pkgver-$arch-unknown-linux-gnu.tar.gz")
sha256sums=('660a06fa13708a37f909a1614b1e830535ebac01f37825ed9b21cd1a489fb5a1')
provides=()
depends=()

package() {
    cd "$srcdir/$pkgname-v$pkgver-$arch-unknown-linux-gnu"
    install -Dm 755 hyprvisor "${pkgdir}/usr/bin/hyprvisor"
}
