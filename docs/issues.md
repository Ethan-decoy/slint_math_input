# Issues and triage

Use the [public issue tracker](https://github.com/Ethan-decoy/slint_math_input/issues).
English and Chinese are welcome. Search for an existing report before opening a new one.
Use one issue for one reproducible defect, user need, or integration question.

## Filing an issue

The issue chooser provides three forms:

| Form | Required information |
| --- | --- |
| Bug | Affected area, exact version and environment, minimal reproduction, expected and actual behavior |
| Feature | User scenario, current limitation, desired observable behavior |
| Question | Goal, version and host setup, attempted approach and documentation |

Keep the generated `[Bug]`, `[Feature]`, or `[Question]` prefix and add a concise description.
For mathematical editing, prefer a short LaTeX expression or a sequence of key/mouse actions.
For embedding, include a minimal host snippet and enabled features. Include screenshots and
sanitized logs when they add evidence; never require a reporter's entire application.

## Labels and ownership

| Label | Meaning |
| --- | --- |
| `bug`, `enhancement`, `question`, `documentation` | Type of work; maintainers may refine the form's initial label |
| `needs-triage` | Awaiting initial maintainer review |
| `confirmed` | Reproduced defect or accepted, sufficiently specified request |
| `in-progress` | A maintainer or contributor is actively working on it |
| `blocked` | Waiting on a named dependency or required information |
| `priority: critical`, `priority: high`, `priority: normal`, `priority: low` | Maintainer-assigned impact and urgency |
| `good first issue`, `help wanted`, `accessibility` | Contributor suitability or affected capability |
| `duplicate`, `invalid`, `wontfix` | Resolution reason; explain it before closing |

Use one active workflow label and at most one priority. After review, replace `needs-triage`
with `confirmed`, `in-progress`, or `blocked` as appropriate. State the missing information
or unblock condition. GitHub's open/closed state supplies the final lifecycle state;
remove active workflow labels when closing. Do not promise response-time guarantees.

## Resolving work

An accepted request should have observable acceptance criteria before implementation.
Link the implementation PR or commit. Use `Closes #number` only for a complete fix,
and `Refs #number` for related or partial work. Close a duplicate with a link to its
canonical issue. Explain declined requests and any viable alternative.

Issue numbers and commit types serve different purposes: a bug report uses `[Bug]`
in its title, while a correcting commit uses `fix(scope): summary`.
See [CONTRIBUTING](../CONTRIBUTING.md) for commit and PR conventions.
