# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 2.0.x   | :white_check_mark: |
| 1.3.x   | :white_check_mark: |
| < 1.3   | :x:                |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to: **security@archivestream.org**

You should receive a response within 48 hours. If for some reason you do not, please follow up via email to ensure we received your original message.

Please include the following information:

- Type of issue (e.g., buffer overflow, SQL injection, cross-site scripting, etc.)
- Full paths of source file(s) related to the manifestation of the issue
- The location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit it

## Disclosure Policy

When we receive a security bug report, we will:

1. Confirm the problem and determine the affected versions
2. Audit code to find any similar problems
3. Prepare fixes for all supported releases
4. Release new security patch versions as soon as possible

## Bug Bounty Program

We currently do not offer a paid bug bounty program, but we will:

- Publicly acknowledge your responsible disclosure (if you wish)
- Add you to our Hall of Fame
- Provide ArchiveStream swag for significant findings

## Security Best Practices

When deploying ArchiveStream:

1. **Use HTTPS**: Always deploy behind TLS/SSL
2. **Secure Database**: Use strong passwords and restrict network access
3. **Update Regularly**: Keep ArchiveStream and dependencies up to date
4. **Limit API Access**: Use rate limiting and API keys for production
5. **Audit Logs**: Enable and monitor access logs
6. **Backup Encryption**: Encrypt backups at rest and in transit

## Known Security Considerations

- **WARC Replay**: Archived JavaScript is sandboxed but may still pose risks. Consider using Content Security Policy headers.
- **Federation**: Only federate with trusted instances. Verify peer signatures.
- **User Input**: All crawl URLs are validated, but be cautious with user-submitted URLs.

## Contact

For general security questions: security@archivestream.org
