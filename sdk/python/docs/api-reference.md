# BCIP Agent SDK — API Reference

Public surface of `openai_codex` for app-server v2.

This SDK surface is experimental. Turn streams are routed by turn ID so one client can consume multiple active turns concurrently.
Thread starts default to `ApprovalMode.auto_review`; turn starts accept an optional `approval_mode` override.

## Package Entry

```python
