# Release Process

This document describes how to create a new release of this Rust project.
It is intended for project maintainers.

`cargo-dist` was used to generate the release scripts for this project. Check
out the `cargo-dist` book for more information:
- https://axodotdev.github.io/cargo-dist/book/introduction.html

---

## Prerequisites

- Maintainer access to the GitHub repository
- Clean working tree
- Stable Rust toolchain installed
- All CI checks passing on `main`

---

## Versioning

This project follows **Semantic Versioning (SemVer)**:

- **MAJOR**: incompatible API changes
- **MINOR**: backwards-compatible features
- **PATCH**: backwards-compatible bug fixes

Versions are prefixed with `v` in Git tags (e.g., `v1.2.3`).

---

## Pre-Release Checklist

Before cutting a release, ensure:

- [ ] All tests pass locally and in CI
- [ ] Documentation is up to date
- [ ] `CHANGELOG.md` contains an entry for the new version
- [ ] No uncommitted changes in the working tree

---

## Release Steps

### 1. Update Version Numbers

Update the version in:

- `fpga_arch_parser/Cargo.toml`
- `fpga_arch_viewer/Cargo.toml`

Example:

```toml
version = "1.2.3"
```

Then run `cargo build` to update the `Cargo.lock` automatically.

### 2. Update the Changelog

Add a new section to `CHANGELOG.md`:

```md
## [v1.2.3] - YYYY-MM-DD
### Added
- ...

### Fixed
- ...
```

### 3. Run Tests

Ensure the project is in a clean state:

```sh
cargo test
```

### 4. Commit the Release

```sh
git commit -am "Release v1.2.3"
```

This can be in a PR. But this commit should be merged into master.

### 5. Create and Push the Git Tag

Make sure you are on a commit of master (i.e. you are tagging a commit that is on the Master branch).

Create a signed tag if possible:

```sh
git fetch origin
git checkout origin/master
git tag v1.2.3
git push --tags
```

Pushing the tag triggers the release workflow.

---

## Post-Release Checklist

- [ ] Verify GitHub release artifacts
- [ ] Announce the release to the VTR community

---

## Rollback Policy

Git tags should **not** be deleted or recreated.

If a release is faulty, issue a follow-up patch release instead.

