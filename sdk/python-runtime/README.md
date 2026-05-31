# BCIP Agent Runtime for Python SDK

Platform-specific runtime package consumed by the published `bcip-agent`.

This package is staged during release so the SDK can pin an exact BCIP Agent
version without checking platform binaries into the repo.

`bcip-agent-cli-bin` is intentionally wheel-only. Do not build or publish an
sdist for this package.
