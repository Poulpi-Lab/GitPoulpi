# Security policy

## Reporting a vulnerability

Please report a vulnerability privately, through GitHub's private vulnerability reporting: open the repository's **Security** tab and choose **Report a vulnerability**. Do not open a public issue or pull request for it.

Include what is affected (version, platform), how to reproduce it, and what an attacker gains. We acknowledge a report, keep you informed while we work on it, and credit you in the notice when you want that.

## Supported versions

Fixes go into the latest release. Older releases are not patched.

## Where notices are published

When a vulnerability is fixed, the maintainers publish a notice:

- as a GitHub security advisory of the repository;
- in `security/advisories.json` of the repository, from which every release carries a signed `security.json` listing the notices (id, affected versions, fixed version, severity, link). The update check of the application reads it.

Release assets are signed with the project's minisign key; the public key is published in the repository.
