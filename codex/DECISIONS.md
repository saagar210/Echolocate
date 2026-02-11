# DECISIONS

## 2026-02-10

1. **Do not use skill-creator / skill-installer**
   - Reason: request is repository remediation/execution, not skill creation or installation.

2. **Proceed despite baseline verification failures**
   - Reason: failures are pre-existing and reproducible before edits; documented as baseline exceptions.

3. **Split alert event types from alert rule types**
   - Reason: backend semantics differ (`unknown_device` event vs `untrusted_device` rule); single union is semantically incorrect.

4. **Monitor uses DB settings each cycle**
   - Reason: enforces source-of-truth invariant and allows live settings updates to take effect without restart.

5. **Do not expand scope to baseline TypeScript cleanup**
   - Reason: user asked to fix latest-commit quality and inline concerns; broad baseline debt is unrelated and would violate small/reversible delta constraint.
