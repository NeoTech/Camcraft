## #1642 - Sign and notarize the macOS release

Use a password-protected Developer ID Application identity and an App Store
Connect team API key for optional macOS distribution signing. The release
workflow passes repository secrets only to the packaging step. The
packager uses a temporary keychain and key file because the hosted runner
has no persistent credential store for this project. A local release can
use the same inputs. If all five inputs are empty, the existing ad hoc
release remains available. A partial configuration fails before packaging.

A workflow step could install and keep a named `notarytool` profile, but
that would move credential setup outside the common packager. The API key
also avoids coupling release automation to a personal Apple Account
password. Changing the authentication method later would require updates
to the signing inputs and the packager.

Submit the app in a temporary ZIP and staple the ticket before making
both final packages. Submit the signed DMG separately so it also has a
stapled ticket. Submitting only the DMG would leave the standalone ZIP
dependent on how its app is taken from the disk image and repackaged. The
two submissions keep both current package formats independent.
