# ark-relations 0.3.0 security patch (NozyWallet)

Upstream rk-relations 0.3 enables optional 	racing-subscriber 0.2 via the std feature.
That crate version is flagged by Dependabot (ANSI log poisoning).

This vendored copy:
- keeps rk-relations 0.3.0 API for rk-groth16 0.3
- removes the 	racing-subscriber dependency
- disables ConstraintLayer / constraint name debug traces (not used by Nozy proving)

Do not bump casually; revisit if ark-groth16 is upgraded past 0.3.
