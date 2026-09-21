# Releasing jgrep

This runbook describes the checks a maintainer should complete before creating
a public release. It does not indicate that a release or release artifact
already exists.

## Prepare the release

1. Select the version and update `CHANGELOG.md` with user-visible changes.
2. Update package metadata, version output, and installation documentation.
3. Confirm that the GPL-3.0-or-later license, `NOTICE`,
   `THIRD_PARTY_NOTICES.md`, bundled license texts, and the generated exact
   dependency inventory are included in source and release archives. The
   release workflow generates that inventory with pinned `cargo about generate`
   against the locked Cargo dependency graph.
4. Run the repository's local CI entry point on a clean checkout. Resolve all
   formatting, lint, test, and documentation-check failures.
5. Review the diff and ensure model files, model caches, credentials, and
   machine-specific build output are absent.

## Build and verify

1. Build the documented target set: macOS Apple Silicon, macOS Intel, Windows
   x64, and Linux x64.
2. Run the CLI smoke tests on every built target, covering file input and
   standard input.
3. Run the pinned `cargo about generate` command and the release packagers, or
   let the tag workflow run them, to create the native archives and the
   versioned installer `.tar.gz`/`.zip` bundles. Verify that native archives
   contain the localized wrappers, marketplace, plugin, licenses, and the
   generated dependency inventory.
4. Produce SHA-256 checksums and independently verify every native archive and
   installer bundle after extraction. The installer bundles must preserve
   hidden `.agents/plugins/marketplace.json` in both archive formats.
5. Do not describe model quality, benchmark results, signing, or notarization
   as available unless the corresponding evidence or artifact is published.

## Publish

1. Create an annotated version tag only after local verification succeeds.
2. Push the release commit and tag, then wait for the GitHub Actions release
   workflow to complete successfully.
3. Let the tag workflow verify every extracted archive and checksum, then
   automatically create the GitHub Release from those verified assets.
4. After publication, inspect the release notes, compatibility requirements,
   known limitations, and installation instructions against the actual assets.
5. Verify the release page, source archive, installer-bundle checksums, and
   one clean installation per supported platform. Exercise a localized wrapper
   from the extracted bundle before referring users to its one-paste command.

If remote CI or artifact verification fails, fix the release candidate and
repeat verification with a new tag as appropriate; do not publish a known-bad
release.
