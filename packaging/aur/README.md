# AUR Packaging for Cedilha

This directory contains the AUR package definitions for Cedilha:
- `cedilha`: Stable release package built from the GitHub release tarball (`v$pkgver.tar.gz`).
- `cedilha-git`: VCS package tracking the latest commit on `master`.

---

## 1. Initial Setup & Publishing to the AUR

1. **Create an AUR account**:
   - Register at [https://aur.archlinux.org/register/](https://aur.archlinux.org/register/).
   - Copy your public SSH key (`cat ~/.ssh/id_ed25519.pub`) and add it to your AUR account under **My Account** -> **SSH Public Key**.

2. **Clone the empty AUR repository**:
   For the stable package:
   ```bash
   git clone ssh://aur@aur.archlinux.org/cedilha.git
   ```
   For the VCS package:
   ```bash
   git clone ssh://aur@aur.archlinux.org/cedilha-git.git
   ```

3. **Copy package files and generate `.SRCINFO`**:
   For `cedilha`:
   ```bash
   cp packaging/aur/cedilha/PKGBUILD cedilha/
   cd cedilha
   makepkg --printsrcinfo > .SRCINFO
   ```
   For `cedilha-git`:
   ```bash
   cp packaging/aur/cedilha-git/PKGBUILD cedilha-git/
   cd cedilha-git
   makepkg --printsrcinfo > .SRCINFO
   ```

4. **Commit and push**:
   ```bash
   git add PKGBUILD .SRCINFO
   git commit -m "Initial import"
   git push origin master
   ```

---

## 2. Bumping Versions (Stable Package)

When releasing a new version (e.g. `0.1.2`):

1. Update `pkgver` and reset `pkgrel=1` in `PKGBUILD`:
   ```bash
   pkgver=0.1.2
   pkgrel=1
   ```

2. Update checksums:
   ```bash
   updpkgsums   # provided by the `pacman-contrib` package
   ```

3. Regenerate `.SRCINFO`:
   ```bash
   makepkg --printsrcinfo > .SRCINFO
   ```

4. Test build locally in a scratch directory:
   ```bash
   makepkg -f
   ```

5. Commit and push to the AUR:
   ```bash
   git add PKGBUILD .SRCINFO
   git commit -m "Update to 0.1.2"
   git push origin master
   ```
