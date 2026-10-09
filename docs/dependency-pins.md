# Dependency pins

> **Doc type:** Living reference — current. Last reviewed: 2026-10-08.

This file records what the pins in `.github/` cannot say themselves: the tag each pinned GitHub Action resolves to, and the version and checksum of every CI tool that a workflow downloads outside Gradle. Gradle dependencies and plugins are pinned elsewhere: in `gradle/libs.versions.toml`, the lockfiles and `gradle/verification-metadata.xml` (S-3 to S-6).

## Rule

- Every `uses:` line in `.github/workflows/` pins an action by its full 40-character commit SHA, never by tag or branch (S-13). The line carries no version comment, because configuration files carry no comments; the table below records the tag instead.
- A pin and its row change in the same commit. A `uses:` line without a row, or a row without a `uses:` line, is drift and is fixed in the commit that notices it.
- Before a SHA is pinned or bumped, check that the tag resolves to it, for example with `git ls-remote https://github.com/<owner>/<repo> 'refs/tags/<tag>*'`. For an annotated tag, pin the commit of the peeled line (`refs/tags/<tag>^{}`), not the tag object. Record the date of the check.
- Dependabot proposes action updates by SHA, after the cooldown of S-7. The pull request updates the row; the reviewer repeats the check above.
- Every `gradle/actions` step sets `cache-provider: basic` or `cache-disabled: true`, because the default cache provider is a proprietary component (S-37). The "Notes" column records which.
- A CI tool that a workflow downloads is pinned by version and by the SHA-256 of the downloaded file, verified before it runs (S-13, S-19, S-36); a Python tool is installed from a hash-locked requirements file. The SHA-256 is taken from the release's own checksum file and checked against the downloaded file.

## GitHub Actions

| Action | Tag | Commit SHA | Checked on | Used in | Notes |
|---|---|---|---|---|---|

No action is pinned yet.

## CI tools

| Tool | Version | Download | SHA-256 | Checked on | Used in |
|---|---|---|---|---|---|

No CI tool is pinned yet.
