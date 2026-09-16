# Contributing to GitPoulpi

Thanks for your interest. Setup, workflow and the pull request checklist are in
[docs/developer/contributing.md](docs/developer/contributing.md); testing rules are in
[docs/developer/testing.md](docs/developer/testing.md).

## Provenance

GitPoulpi is written from scratch. Its design takes ideas from other git clients, but only from public
information: their websites, their public documentation and their observable behavior. No code, binary,
asset (icon, image, font, theme file), text or data file of another product has been used, and no other
product has been decompiled, unpacked or reverse-engineered.

Contributions must follow the same rule:

- Do not submit code, assets, strings, keyboard maps or data copied from another product, and do not
  submit anything obtained by decompiling, unpacking or inspecting the internals of another application.
- Describe features in your own words. Short generic labels that every git client uses ("Push", "Pull",
  "Stash") are fine; longer texts such as help paragraphs must be your own.
- Third-party open source code is welcome only under a license compatible with MIT, and it must be listed
  in `THIRD-PARTY-LICENSES.txt` (`cargo run -p xtask -- licenses` regenerates it from `Cargo.lock`).

Product names are trademarks of their owners. GitPoulpi is an independent project and is not affiliated with,
endorsed or sponsored by any of them.

## Tags

Tags are prefixed by purpose so the top level stays clean:

- `releases/vX.Y.Z` (and `releases/vX.Y.Z-rc.N` for a pre-release): a published version. Pushing one runs
  `.github/workflows/release.yml`; nothing else triggers a release.
- `backups/<name>`: a landmark kept to come back to (before a large refactor, a migration wave...).
- `dryrun/vX.Y.Z[-rc.N]`: a release rehearsal. Pushing one alone triggers nothing; run
  `gh workflow run release.yml -f tag=dryrun/vX.Y.Z -f dry_run=true` to build every archive and the installer
  without publishing. Delete the tag afterwards.

Add a new prefix for a new purpose rather than a bare tag.

## License

By contributing, you agree that your contributions are licensed under the [MIT License](LICENSE), like the
rest of the project.
