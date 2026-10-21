<!-- Copy this template when preparing a Release; GitHub does not apply it automatically. -->
<!-- Follow docs/releases.md. Fill in verified facts and remove these comments. -->

## Highlights

Explain the user-visible outcome and intended audience in two or three bullets.

## Changes

Group relevant changes under Added, Changed, Fixed, or Removed. Link resolved issues and PRs.
For the first release, describe the delivered capabilities instead of inventing a comparison.

## Compatibility and integration

State the Rust/Slint requirements, API or draft-schema changes, migration steps, and release channel.
Describe typing/navigation changes even when the Rust interface is unchanged.
Include a usable dependency or checkout example with the actual version; align runtime and build dependencies when both are used.

## Verification

Link CI for the exact tagged commit and list the checks that passed.
Identify the host/example, candidate revision, affected scenarios, environment, and acceptance result.
Distinguish offscreen integration tests from native desktop verification.

## Known limitations

State material unsupported or unverified behavior and link the support table.
Identify downstream or native checks that remain unperformed.

## Downloads and integrity

Describe the source archives and any binary assets that are actually attached.
Identify the exact public source commit and tag, and state which downloaded contents were verified.
For custom assets, include their source commit, target platform, license notices, and SHA-256 checksums.

## References

Link the version's changelog, source commit, integration guide, and full comparison when there is a previous release.
