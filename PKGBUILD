# Maintainer: Kien H. Luu <kien.luuhoang.arch@gmail.com>

pkgname=hyprvisor
pkgver=0.4.4
pkgrel=1
pkgdesc="Hyprvisor is a backend watcher for Hyprland."
arch=("x86_64")
url="https://github.com/lulkien/hyprvisor"
license=('UNLICENSE')
source=("$pkgname-v$pkgver-$arch-unknown-linux-gnu.tar.gz::$url/releases/download/v$pkgver/$pkgname-v$pkgver-$arch-unknown-linux-gnu.tar.gz")
sha256sums=('8f39935fe49a811b8341b6c048c7d12a6e0882e1afeeadf0a2b790488b246b32')
provides=()
depends=('hyprland' 'bluez' 'iwd' 'dbus')

package() {
    cd "$srcdir/$pkgname-v$pkgver-$arch-unknown-linux-gnu"

    install -Dm 755 hyprvisor "${pkgdir}/usr/bin/hyprvisor"

    install -Dm 644 hyprvisor.service "${pkgdir}/usr/lib/systemd/user/hyprvisor.service"
}
