# Release policy

SMI uses [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html)
and a curated [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) history.
This policy applies to every preview and stable release. It is the authoritative
release checklist; the [release template](../.github/RELEASE_TEMPLATE.md) supplies
the announcement format. Versioned source releases belong to the public repository.

## Version and channel

- Match the package version in `Cargo.toml` and `Cargo.lock`, the README status,
  the changelog heading, and the `v`-prefixed annotated tag.
- Use increasing prerelease identifiers, such as `0.1.0-preview.1` and
  `0.1.0-preview.2`. Mark these GitHub releases as prereleases, not Latest.
- During `0.x`, document public API and schema compatibility explicitly.
  A `1.0.0` release requires a defined stable public interface.
- Include editing-behavior changes in compatibility guidance even when Rust
  method signatures and the draft schema are unchanged.
- `publish = false` means a GitHub source release is not a crates.io release.
  Changing registry publication requires a separate release decision.

## Required release sequence

1. **Define the release.** Review changes since the previous version, choose the
   version/channel, and document user impact and migration. Move the delivered
   changes from `Unreleased` into a dated `YYYY-MM-DD` changelog section. Leave
   unrelated pending work under `Unreleased`.
2. **Freeze and validate the public candidate.** Record the exact, clean public
   commit. Run the [contributor checks](../CONTRIBUTING.md#build-and-checks), inspect
   the generated source package, and complete the host checks described below.
   Wait for every required CI job on that same commit, including supported OSes,
   MSRV, quality/package checks, and commit conventions. A queued, running, or
   failed job does not satisfy this gate. A changed candidate needs checks for
   the new commit.
3. **Tag the verified source.** Create an annotated version tag on that commit.
   Verify the tag resolves to the checked source and that version and license
   material agree with the candidate.
4. **Review the draft delivery.** Prepare a **Draft Release** using the template.
   Review rendered notes, compatibility guidance, verification links, and actual
   downloads. Download a tagged source archive and compare its files with the
   public commit; verify custom assets as described below. The reviewed draft,
   tag, and downloads must all identify the same source.
   Draft status does not hide source commits already pushed to a public repository.
5. **Publish and read back.** Complete publication under the maintainer's existing
   authorization once the gates pass. Verify the published version, source commit,
   draft/prerelease/Latest state, notes, and download links. Keep previews marked
   as prereleases and explicitly excluded from Latest.
6. **Record and follow up.** Retain the source/tag references, CI results, package
   and host-check evidence, release URL, and remaining limitations. Reproduce
   downstream reports against the stated version, add behavior regressions, and
   deliver source corrections as new versions. Track reports through the
   [issue workflow](issues.md).

## Verification evidence

Validate the distributable contents, not only the working checkout. Use
`cargo package --locked` while registry publication is disabled; inspect the
archive and retain its build-verification result. Include the intended source,
tests, examples, and licenses while excluding internal maintenance files and
temporary build output. A separately authorized registry release also requires
`cargo publish --dry-run` and the registry-specific preparation.

For editing or integration behavior changes, run a focused acceptance check using
the exact public candidate in a host. Cover the affected inputs and relevant
navigation, clipboard, draft restoration, and submission behavior. Record the
host, candidate revision, scenarios, environment, and outcome. An independent
maintained embedding example can serve as the host when a downstream application
is unavailable; state that limitation and the remaining downstream checks.

Keep automated/offscreen evidence separate from real desktop verification. State
which native environments were exercised and which remain unverified; align
support claims with [the support table](support.md). An unperformed host check
must not be reported as passed.

## Announcement

Use `v<version> — <short outcome>` as the title. Release notes explain user outcomes,
integration, compatibility, exact validation, known limitations, and downloads.
Dependency examples must use the released tag or public commit, keeping runtime
and build dependencies aligned when both are used.
Do not copy raw commit logs or internal development history into the announcement.
GitHub's generated notes use [category configuration](../.github/release.yml)
as a starting point; maintainers still curate the result.

## Source and binary downloads

GitHub provides source archives for the tagged public commit. List custom binary
assets only when they exist and have been tested from their extracted directory.
Each custom package must identify its public source commit and target platform,
carry the required license notices, and have a published SHA-256 checksum.

Source is the default deliverable for this library. Add demonstration binaries
when they serve an actual delivery need, with their supported target and tested
contents stated explicitly.

## Corrections and published tags

Keep published version contents and tags stable. Ship source changes under a new
version; editing release prose is allowed when it corrects or clarifies existing facts.
Do not routinely force-push shared history to improve wording. An expressly
authorized history repair requires a verified backup, recorded old/new references,
and `--force-with-lease` against the inspected remote values. If a release reference
must be repaired, verify its source tree is unchanged and disclose the correction
in the release notes; preserve the version's source contents.

## Keeping the process repeatable

Use the same release sequence each time. Automate repeated checks and preparation
where they remove demonstrated omissions; keep the same gates when executing
steps manually. A planned script or workflow is not an implemented release tool.
Changing repository protection settings or enabling registry publication is a
separate task from following this policy.
