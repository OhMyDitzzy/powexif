# Security Policy

## Reporting a Vulnerability

If you discover a security issue in powexif — such as a parsing bug that could
cause crashes, out-of-bounds reads, or malicious behavior when processing
untrusted image files — please **do not open a public GitHub issue**.

Instead, report it privately by emailing the maintainer at the address shown
on the GitHub profile, or by using
[GitHub's private vulnerability reporting](https://github.com/OhMyDitzzy/powexif/security/advisories/new).

Please include:

- A description of the vulnerability
- Steps to reproduce (a minimal test file or hex dump if possible)
- The version of powexif affected
- Your assessment of the severity and impact

You can expect an acknowledgement within **72 hours** and a fix or public
disclosure plan within **14 days**, depending on complexity.

## Scope

powexif parses JPEG and TIFF files, which are commonly received from
untrusted sources (user uploads, web scraping, camera imports). Bugs in
the parser that could be triggered by a crafted file — such as integer
overflows, infinite loops, or excessive memory allocation — are in scope.

Bugs that only affect the CLI tool's user-facing output format are
generally lower severity and can be reported as normal issues.