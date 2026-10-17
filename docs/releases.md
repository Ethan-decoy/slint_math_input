# Release policy

SMI uses [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html)
and a curated [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) history.
Versioned source releases belong to the public repository.

## Version and channel

- Match the Cargo version, changelog heading, and `v`-prefixed annotated tag.
- Use increasing prerelease identifiers, such as `0.1.0-preview.1` and
  `0.1.0-preview.2`. Mark these GitHub releases as prereleases, not Latest.
- During `0.x`, document public API and schema compatibility explicitly.
  A `1.0.0` release requires a defined stable public interface.
- `publish = false` means a GitHub source release is not a crates.io release.
  Changing registry publication requires a separate release decision.

## Preparation and checks

1. Review the user-visible changes and update `CHANGELOG.md`. Keep pending changes
   under `Unreleased`; date released versions using `YYYY-MM-DD`.
2. Validate the clean public snapshot using the contributor checks. Wait for CI
   on the exact release commit, including all supported OS jobs and MSRV.
3. Create an annotated version tag on that commit. Verify its source, version,
   license material, and any custom asset checksums.
4. Prepare a **Draft Release** using the [release template](../.github/RELEASE_TEMPLATE.md).
   Draft status does not hide source commits already pushed to a public repository.
5. Review the rendered notes, links, compatibility guidance, verification evidence,
   and actual downloadable files before publication. Follow the maintainer's
   authorization for publishing the release.

Use `v<version> — <short outcome>` as the title. Release notes explain user outcomes,
integration, compatibility, exact validation, known limitations, and downloads.
Do not copy raw commit logs or internal development history into the announcement.
GitHub's generated notes use [category configuration](../.github/release.yml)
as a starting point; maintainers still curate the result.

## Source and binary downloads

GitHub provides source archives for the tagged public commit. List custom binary
assets only when they exist and have been tested from their extracted directory.
Each custom package must identify its public source commit and target platform,
carry the required license notices, and have a published SHA-256 checksum.
Do not claim native IME, accessibility, or device testing from offscreen CI alone.

## Corrections and published tags

Keep published version contents and tags stable. Ship source changes under a new
version; editing release prose is allowed when it corrects or clarifies existing facts.
Do not routinely force-push shared history to improve wording. An expressly
authorized history repair requires a verified backup, recorded old/new references,
and `--force-with-lease` against the inspected remote values. If a release reference
must be repaired, verify its source tree is unchanged and disclose the correction
in the release notes; preserve the version's source contents.
