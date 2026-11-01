# Security policy

Flourish Core is a public, MIT-licensed Rust domain library. **Do not disclose suspected security vulnerabilities in public issues, pull requests or discussions.**

## Report a vulnerability privately

Use GitHub's [private vulnerability reporting form](https://github.com/Sven-Lutz/flourish-core/security/advisories/new). If needed, open the repository's **Security** tab and choose **Report a vulnerability**.

This channel is intended for confidential reports. Do not attach actual user journals, credentials, access tokens, private records or data obtained without authorization.

The repository owner (`@Sven-Lutz`) is the initial triage contact through GitHub's repository-managed security reporting. **No response-time SLA or support contract is promised.** If the reporting form is unavailable, do not post exploit details publicly; GitHub's reporting controls must be restored before such details are shared.

A helpful report contains the affected commit or version, conditions needed to reproduce, the trust boundary affected, observed vs expected behavior, potential impact, and minimal synthetic reproduction steps. Do not test against other people's systems or access private information.

## Supported versions

The initial public state is a **source-only, development-stage snapshot**, with `version = "0.1.0"` in `Cargo.toml`. There is no tagged release, published crates.io artifact, or declared long-term-support version. Security fixes, if needed, will be published through normal reviewed source updates. Do not interpret the package version as a guarantee of ongoing support.

The desktop/mobile Flourish applications, storage integrations and hosted services are **not part of this repository's security scope**.

## Handling

Reports should be triaged privately, reproduced with non-sensitive data, fixed on a restricted branch if necessary, and disclosed in coordination with the reporter. Rotate any real compromised secrets promptly. If the issue affects a third-party dependency, coordinate with the appropriate upstream maintainer.

This policy does not claim that the library has undergone an independent security audit.
