# Maintainer: Kien H. Luu <kien.luuhoang.arch@gmail.com>

pkgname=hyprvisor
pkgver=0.4.4
pkgrel=1
pkgdesc="Hyprvisor is a backend watcher for Hyprland."
arch=("x86_64")
url="https://github.com/lulkien/hyprvisor"
license=('UNLICENSE')
source=("$pkgname-v$pkgver-$arch-unknown-linux-gnu.tar.gz::$url/releases/download/v$pkgver/$pkgname-v$pkgver-$arch-unknown-linux-gnu.tar.gz")
sha256sums=('c957e87f95b574c4e9da2b4d53a93b848596256071abebc3a6edb5e36fc4e01d')
provides=()
depends=('hyprland' 'bluez' 'iwd' 'dbus')

package() {
    cd "$srcdir/$pkgname-v$pkgver-$arch-unknown-linux-gnu"

    install -Dm 755 hyprvisor "${pkgdir}/usr/bin/hyprvisor"

    install -Dm 644 hyprvisor.service "${pkgdir}/usr/lib/systemd/user/hyprvisor.service"
}
