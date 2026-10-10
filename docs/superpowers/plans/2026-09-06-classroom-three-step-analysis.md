# Classroom Three-Step Analysis Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Every natal chart receives explainable classroom pattern, strength, and conditional preference analysis without changing existing quantitative results.

**Architecture:** A pure Rust lesson module computes facts and three-step candidates. The existing lesson response gains an optional typed result and Vue renders it above collapsible legacy annotations. References never feed into classroom decisions.

**Tech Stack:** Rust, tyme4rs, serde, Tauri 2, Vue 3, TypeScript, Element Plus; existing Vite and Node test runner, no new dependencies.

**Spec:** `docs/superpowers/specs/2026-09-06-classroom-three-step-analysis-design.md` (approved by user).

## Global Constraints

- Version `classroom-2026-09-v1`; all conclusions remain candidate/conditional/insufficient. Engineering support threshold is explicit and never promoted to teacher certainty.
- Keep normal geJu, quantitative scores, original use gods, special patterns, luck and existing exact-case annotation behavior intact.
- Same four pillars yield identical classroom core regardless of date/direct entry and reference values. Comparison text may differ.
- No network LLM, new dependency, broad refactor, birth-time adjustment, event predictions, or rules-library UI.
- Do not create or modify fixtures for unclear course charts. Structural synthetic charts must be labeled as such.
- Preserve pre-existing uncommitted edits. Agents do not commit or stage overlapping files; review current changes against controller's pre-task baseline, not HEAD. No push/merge.
- Work on `codex/classroom-three-step-analysis` in the current directory, preserving the working tree on which the approved spec is based.
- Read the spec and applicable project skill before implementing. Exact source chart lookup must never replace generalized three-step rules.

## Task 1: Rust three-step module, response integration, and behavioral tests

**Owner/model:** gpt-5.6-sol, high. **Review:** gpt-6-astra, high.

**Files:**
- Create `src-tauri/src/lesson_analysis.rs`, optionally focused `src-tauri/src/lesson_analysis/{facts,pattern,strength,preferences,tests}.rs` when needed to avoid a monolith.
- Modify `src-tauri/src/lib.rs` to register module.
- Modify `src-tauri/src/bazi.rs` only for response field and integration tests.
- Modify `src-tauri/src/quant_model.rs` only for three-step call/flow text and intentional legacy test compatibility.

**Interfaces:**
```rust
pub(crate) fn analyze(
    natal_cycles: &[SixtyCycle; 4],
    reference: &LessonModelReference,
    interactions: &[QuantModelInteraction],
) -> LessonThreeStepAnalysis;
```
`LessonModelReference` contains original ge_ju, strength_label, use_elements and optional original special pattern/category as strings. The module owns its serializable DTOs with camelCase names. `QuantLessonContent.three_step_analysis: Option<LessonThreeStepAnalysis>` serializes as `threeStepAnalysis`.

The module must implement all spec facts/pattern/strength/preferences/comparison fields and named evidence kinds. Define concrete DTOs in Rust first; include their complete camelCase contract in the task report for Task 2. Return sources and limitations separately from conclusions. Source dates retain exact ISO strings.

- [x] Write a failing integration assertion via serde_json on an existing `analyze_pillars` response before adding the field. The following assertion must fail because the new output is absent:
```rust
let json = serde_json::to_value(response).unwrap();
assert_eq!(json["quantModel"]["lessonContent"]["threeStepAnalysis"]["version"],
           "classroom-2026-09-v1");
```
Run `cargo test --manifest-path src-tauri/Cargo.toml three_step -- --nocapture` and save the expected assertion failure in the report.

- [x] Implement input facts using tyme4rs's actual stems/branches/hidden stems/ten-star APIs or existing shared helpers. Do not duplicate lookup tables. Each root retains pillar and hidden-stem layer. Every factual explanation is derived from input.
- [x] Implement ordinary pattern candidates: exclude day-stem transparency; unique eligible transparent candidate selected, multiple retained as candidates, none falls back to month hidden order. Respect the spec's distinction between normal result and classroom candidates; direct input never shows `直輸四柱分析` as classroom pattern.
- [x] Implement strength with literal support threshold from spec, strict same-season separate from seal-in-season, surface pressure families separate from hidden presence. Zero factors and incomplete three-family pressure is insufficient, not extreme weak. Interactions preserve actual IDs/positions and mark conditional rather than applying new score/pruning logic.
- [x] Implement all eight ordinary preference branches, structured family-to-element directions, per-branch conditions/rejections. Root and surface evidence must be linked. No priority by first match and no cross-scenario union of conflicting roles. Third step consumes classroom facts and strength only; reference cannot affect its core.
- [x] Implement special-pattern limitations including independent extreme/rootless trigger, two date-tagged specialized-use versions and follow-child peer exception. These are conditional scope explanations, not unverified new classifiers. Emit ordinary plans only under an explicit ordinary-pattern condition when special status is unresolved.
- [x] Implement five bridge mappings with actual nodes and missing intermediates, medicine only attached to a plan's protected role, and bounded seasonal compatibility notes. When unassessed, explain why rather than claiming success.
- [x] Add behavioral tests with literal independently derived expectations. Include:
```rust
// Metadata-only references cannot change the classroom core.
assert_eq!(serde_json::to_value(a.facts).unwrap(), serde_json::to_value(b.facts).unwrap());
assert_eq!(serde_json::to_value(a.strength).unwrap(), serde_json::to_value(b.strength).unwrap());
assert_eq!(serde_json::to_value(a.preferences).unwrap(), serde_json::to_value(b.preferences).unwrap());
```
Test both production entrypoints, near-match nonlookup behavior, same-season vs seal-in-season, support boundary, zero-factor partial/full pressure, roots with hidden opposing elements, multi-transparency, competing plans, distinct wealth/kill locations, special guards, no fixed winter-fire preference, and backward-compatible old model outputs. Use real calls, not grep tests or self-derived expected values.
- [x] Update `analysis_order` to `["格局判定", "日主強弱", "用神與喜忌"]` and its existing test. Keep old inferences intact; update source range text to 09/02 without claiming all course rules are verified.
- [x] Run focused tests, then full Rust tests once. Write full RED/GREEN evidence, concrete contract, files and concerns to task-1-report.md. Do not commit dirty overlapping files.

## Task 2: Three-step UI, TypeScript, rendered component tests

**Owner/model:** gpt-5.6-luna, medium; upgrade to sol if prose-to-contract work proves non-mechanical. **Review:** gpt-5.6-sol, high.

**Files:**
- Create `src/components/LessonThreeStepPanel.vue`.
- Modify `src/components/CourseContentPanel.vue`, `src/types/bazi.ts`.
- Create `tests/lesson-three-step-ui.test.mjs` plus test-only fixtures if needed.
- Add `test:lesson-ui` to package.json without changing dependencies or version.

**Interfaces:** Consume the exact camelCase DTO from Task 1's report/source. No frontend inference. Component prop `analysis?: LessonThreeStepAnalysis | null`.

- [x] Add a Node rendered-component test using existing Vite SSR and `vue/server-renderer`, before adding the component:
```javascript
import test from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { createSSRApp, h } from 'vue';
import { renderToString } from 'vue/server-renderer';
// Load real .vue using server.ssrLoadModule; install ElementPlus if used.
// Render no-analysis state and complete real backend fixture.
// Assert visible step headings and independently known fixture facts.
```
Run `node --test tests/lesson-three-step-ui.test.mjs` and record the pre-implementation missing component/failing behavior; then implement and rerun.
- [x] Build simple layout: ordered three sections with conclusion, per-step important evidence, collapsible details for proxy thresholds and IDs. Use `<details>`/`<summary>` or existing ElementPlus, not new UI dependencies. Preserve long Chinese line wrapping.
- [x] Display alternatives as separate articles/cards with use/favorable/unfavorable/conditional directions. Keep special and ordinary conditions visible; never label candidate as verified/success.
- [x] For absent analysis show `此結果尚無三步分析`; old course data still renders. In CourseContentPanel insert the new panel above legacy annotations and wrap legacy list in `課堂命例補充` details. Do not add rules library. Existing normal panels and positions unchanged.
- [x] Tests render actual Vue components with literal fixtures matching backend serialization. Cover no result, one and multiple preference branches, conditional/insufficient states, engineering/source details, old annotations still reachable, escaping untrusted text, and all three steps. Do not assert mock presence or source contains strings.
- [x] Run `npm run test:lesson-ui`, `npm run build`; report RED/GREEN and contract choices to task-2-report.md. Do not stage/commit overlapping files.

## Task 3: End-to-end verification, review, documentation

**Owner:** controller, with Astra final review and one scoped fixer if required.

**Files:** update approved spec implementation status and root `上課筆記整理.md` engineering section to reflect actual delivery; preserve course material and all uncertainty.

- [x] Compare new serialization of at least one general non-source chart and its one-pillar variant; retain output for real UI checks. No golden answer computed with the function under test.
- [x] Run full Rust, Node rendered-component tests and production build. Baseline is 27 Rust tests passing, pre-existing Rust dead-code warnings and Vite chunk-size warning. Do not fix unrelated baseline warnings.
- [x] Start a loopback-only local dev UI; use a test-only fixture page if browser has no Tauri IPC, explicitly label it as fixture evidence. Check desktop and narrow widths, details expansion, multiple plans, old data fallback and no rules library. Do not claim native Tauri launch if only browser fixture checked.
- [x] Package only this task's changes versus captured working-tree baseline, dispatch final independent Astra review. Fix load-bearing findings and rerun covering tests plus final relevant validations.
- [x] Update doc delivery state with real test results, limitations and preserved original fields. Main agent reports exact changes, verified scope and remaining conditional judgment.
- [x] Keep app changes uncommitted unless a scoped valid commit can be separated from pre-existing dependent edits; do not manufacture a broken selective commit. No automatic merge/push. Review and final status must make this boundary explicit.
