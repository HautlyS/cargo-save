# Security Policy

## Supported Versions

The following versions of cargo-save are currently supported with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |
| < 0.2.0 | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in cargo-save, please report it responsibly.

### How to Report

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please send an email to hautlythird@gmail.com with:

- A description of the vulnerability
- Steps to reproduce the issue
- Possible impact of the vulnerability
- Any suggestions for mitigation or fixes

Please include "SECURITY" in the subject line.

### What to Expect

- **Acknowledgment**: We will acknowledge receipt of your report within 48 hours
- **Assessment**: We will assess the vulnerability and determine its impact
- **Updates**: We will keep you informed about our progress
- **Resolution**: Once fixed, we will notify you and publicly acknowledge your contribution (unless you prefer to remain anonymous)

### Response Timeline

- **Initial Response**: Within 48 hours
- **Assessment Complete**: Within 7 days
- **Fix Released**: Depends on severity
  - Critical: Within 7 days
  - High: Within 30 days
  - Medium/Low: Next scheduled release

## Security Considerations

### Cache Security

- Cache files are stored in the user's cache directory
- Cache files contain build output which may include sensitive information
- Do not share cache directories between untrusted users
- Cache invalidation is based on content hashes, not file permissions

### Git Security

- cargo-save executes git commands to detect changes
- Only works within git repositories you trust
- Does not execute arbitrary code from the repository
- Git hooks installed by cargo-save only invalidate caches

### Environment Variables

- cargo-save reads certain environment variables that affect builds
- These are hashed into the cache key for correctness
- See [ENV_VARS_THAT_AFFECT_BUILD] for the full list

### Safe Usage

- Only use cargo-save with code you trust
- Be cautious when caching builds of dependencies
- Regularly clean old caches with `cargo save clean`
- Do not commit cache files to version control

## Security Best Practices

### For Users

1. Keep cargo-save updated to the latest version
2. Use `cargo save clean` regularly to remove old caches
3. Be aware of what environment variables affect your builds
4. Don't share cache directories between different security contexts
5. Review git hooks installed by `cargo save install-hooks`

### For Developers

1. Follow secure coding practices
2. Avoid executing user input as commands
3. Validate all file paths before access
4. Use proper error handling to avoid information leakage
5. Keep dependencies updated

## Disclosure Policy

When we receive a security report, we will:

1. Confirm the issue and determine its severity
2. Develop a fix and test it
3. Prepare a security advisory
4. Release the fix and publish the advisory simultaneously
5. Credit the reporter (unless anonymous)

## Past Security Issues

None reported yet.

## Acknowledgments

We thank the following individuals for responsibly disclosing security issues:

*None yet - be the first!*
