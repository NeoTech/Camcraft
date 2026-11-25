## #3773 - The Arch package reuses the Linux release binaries of the same run

**What was decided.** `.github/workflows/aur-package.yml` no longer compiles
CamCraft. It downloads the `linux-packages` artifact that the `linux` job of
the same `release.yml` run uploaded, unpacks the binaries `camcraft-ui`,
`camcraft-cli` and `post` from the `camcraft-<version>-linux-x86_64.tar.gz`
tarball, and packages them with the AUR recipe:

- `makepkg -o --noprepare -d` downloads, checks and extracts the source of
  the recipe (on a tag run, the GitHub source tarball of the tag). A dispatch
  run replaces that tree with its own checkout, because its binaries come
  from that checkout.
- The binaries go to `target/release/` of the source tree.
- `makepkg -R -d` runs only `package()`.
- A CI copy of the recipe (`PKGBUILD.prebuilt`, never committed) adds
  `'!strip' '!debug'` to `options`. makepkg then does not rewrite the
  binaries.
- A step compares the SHA256 of each packaged binary with the binary in the
  Linux tarball, and fails the job on a difference. On a tag run, the
  tarball version must agree with the tag.

`aur/PKGBUILD` and `aur/.SRCINFO` stay the from-source recipe. The AUR
guidelines expect a package named `camcraft` (without `-bin`) to build from
source, so the recipe that AUR users run does not change.

**Why.** The maintainer asked for it (2026-11-23): the second build cost
time and billed minutes. In the v0.25.0 tag run (45269109413) the `aur /
build` job ran from 21:42:18 to 22:31:11, 48 min 53 s, and `publish` waited
for it 30 minutes after the last other platform (macOS, 22:01:16). The new
job installs only `base-devel` and `curl` and compiles nothing.

**Considered and dropped.**
- Download the release asset of the tag: the asset exists only after
  `publish`, which waits for this job. The artifact of the same run is
  available at once and also works on a dispatch run.
- A separate `camcraft-bin` recipe in the repository: a second recipe to
  keep in step, and the issue asks only to stop the second build.
- Keep makepkg's strip: the packaged binary is then not the release binary,
  and the SHA256 check cannot prove anything.

**Cost to change later.** The Arch package now holds binaries built on
`ubuntu-22.04` (glibc 2.35), which run on the newer glibc of Arch. If the
Arch package must be built against Arch's own libraries again, restore the
`cargo build` step (about 45 billed minutes for each tag run).
