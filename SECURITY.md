# Security Policy

## Supported Versions

We release patches for security vulnerabilities in the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security vulnerability in Polis, please report it to us as described below.

### How to Report

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via one of the following methods:

1. **Email**: Send details to [security@polis.dev](mailto:security@polis.dev)
2. **GitHub Security Advisories**: Use the [GitHub Security Advisories](https://github.com/polis-project/polis/security/advisories) feature
3. **Private Issue**: Create a private issue with the "security" label

### What to Include

Please include the following information in your report:

- Description of the vulnerability
- Steps to reproduce the issue
- Potential impact
- Suggested fix (if any)
- Your contact information (for follow-up questions)

### Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 7 days
- **Resolution**: Depends on severity and complexity

### Severity Levels

We use the following severity levels:

- **Critical**: Remote code execution, privilege escalation, data breach
- **High**: Significant security impact, but not critical
- **Medium**: Moderate security impact
- **Low**: Minor security impact

### Disclosure Process

1. **Report**: Vulnerability is reported privately
2. **Acknowledge**: We acknowledge receipt within 48 hours
3. **Investigate**: We investigate and assess the vulnerability
4. **Fix**: We develop and test a fix
5. **Release**: We release the fix in a security update
6. **Disclose**: We publicly disclose the vulnerability after the fix is available

### Responsible Disclosure

We follow responsible disclosure practices:

- We will not publicly disclose vulnerabilities until a fix is available
- We will credit security researchers who report vulnerabilities
- We will work with researchers to ensure proper disclosure
- We will provide regular updates on the status of reported vulnerabilities

### Security Best Practices

To help keep Polis secure, please:

- Keep your Polis installation up to date
- Use strong authentication
- Follow security best practices
- Report security issues responsibly
- Participate in security discussions

### Security Features

Polis includes several security features:

- **Namespaces**: Process isolation using Linux namespaces
- **Control Groups**: Resource limits and quotas
- **Seccomp**: System call filtering
- **Capabilities**: Fine-grained permission management
- **AppArmor**: Mandatory access control
- **SELinux**: Security-enhanced Linux support
- **Image Scanning**: Vulnerability detection in container images
- **Network Security**: Firewall rules and network policies
- **Authentication**: JWT-based authentication
- **Authorization**: Role-based access control

### Security Updates

Security updates are released as needed. We recommend:

- Enabling automatic updates where possible
- Monitoring security advisories
- Testing updates in non-production environments
- Keeping backups of important data

### Contact Information

For security-related questions or concerns:

- **Email**: [security@polis.dev](mailto:security@polis.dev)
- **GitHub**: [Security Advisories](https://github.com/polis-project/polis/security/advisories)
- **Discord**: [Security Channel](https://discord.gg/polis)

### Acknowledgments

We thank the security community for their contributions to making Polis more secure.

### Legal

This security policy is subject to our [Terms of Service](TERMS.md) and [Privacy Policy](PRIVACY.md).
