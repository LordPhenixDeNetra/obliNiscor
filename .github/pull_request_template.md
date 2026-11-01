## Task and requirements

<!--
What was asked, and what "done" means for it. Name the task (OBL-*) and the requirements it
covers (FR-*, NFR-*, SEC-*). One task per pull request.
-->

## What changed

<!--
The pieces, in the order a reviewer should read them. Name the decision made, if any, and
point at its ADR.
-->

## Tests

<!--
Which tests prove the behaviour, and why they would fail without this change. For a system
call, say which privilege was used, on which system and version, and what happens without it.
-->

## Checks run

- [ ] `cargo xtask ci` — exit 0 on Windows, Linux and macOS
- [ ] The end-to-end or `#[ignore]`d tests for the touched system were run, and their result
      is recorded in `docs/spikes/` or `docs/mesures/`
- [ ] The task's acceptance criterion is demonstrated (measurement, log, replayable command),
      and the proof is linked in the tracking document

## What is not done

<!--
Known gaps, each with the reason. Also: what is unavailable on some system, and how the
interface says so.
-->

## Checklist

- [ ] The task `OBL-*` is cited, and the tracking document is updated
- [ ] No `unsafe` outside `obliniscor-sys`; every `unsafe` block has a `SAFETY:` comment
- [ ] Every durable effect (deletion, network rule, service change) is logged **before** it
      happens
- [ ] A function unavailable on a system is declared, not simulated (matrix 6.0)
- [ ] Every visible string is in `locales/en` **and** `locales/fr`
- [ ] No code copied from a GPL project (ADR-001, clean room); sources cited
- [ ] Commits are signed off (`git commit -s`) and carry no tool attribution
