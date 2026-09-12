# 🛡️ Security Policy

**What snora's attack surface is** — and what it is not — is documented in the
[threat model](https://nabbisen.github.io/snora/reference/threat-model.html).
Reading it first may save you time: snora is a library with no network, storage
or privilege boundary, and the one category where a snora defect becomes an
application's vulnerability is overlay input containment.

## Reporting a vulnerability

Please **do not** open a public GitHub issue for a suspected security
problem. Instead, please report it privately using GitHub's security advisory feature:

> https://github.com/nabbisen/snora/security/advisories/new

Please include:

- A description of the issue and the affected crate(s).
- The version, commit hash, or branch the report applies to.
- A reproducer if at all possible: a minimal application, a `cargo test` invocation, or the sequence of interactions that demonstrates the issue.
- Your assessment of the impact.

We aim to acknowledge any report within seven days, and to publish a
fix or a clear timeline within thirty days of acknowledgement.

### Supported Versions

This project is currently maintained as a best-effort activity. Only the latest version is considered to receive potential fixes.
