# How to Contribute

Contributions are welcome! Here is how to contribute.

## Contribution Guidelines

Please read the [contribution guidelines](https://github.com/timothee-haudebourg/admin/blob/main/files/CONTRIBUTING.md) before submitting your changes. Contributions that do not adhere to these guidelines may be rejected without further explanation.

## Release Process

- Bump the version number in `Cargo.toml`;
- Make a commit with the message `chore(release): Version X.Y.Z`;
- Publish the crate;
- On success, add a tag with `git tag -a vX.Y.Z -m "Version X.Y.Z"` and push.