# Ge Ju Month-Branch Transparency Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make 格局判定 explain and resolve multiple 月令藏干透干 candidates by strength.

**Architecture:** Keep logic inside `src-tauri/src/bazi.rs`, because current 格局判定 and tests already live there. Add a small candidate model that pairs 月令藏干十神 with 藏干層級 and 天干透出位置, then rank candidates before producing `ge_ju` and `ge_ju_basis`.

**Tech Stack:** Rust backend in Tauri, existing `cargo test`, existing Vue frontend consumes `geJuBasis` unchanged.

## Global Constraints

- Do not change TypeScript or Rust public response fields.
- Do not change frontend display contract: `SummaryPanel.vue` continues to render `result.geJuBasis`.
- Keep rule deterministic and explainable: 月令藏干透干優先；多個透干時先比本氣/中氣/餘氣，再比透干位置（月干 > 時干 > 年干）。
- Preserve direct pillar flow and existing build behavior.

---

### Task 1: Add failing tests for detailed basis and multiple transparent candidates

**Files:**
- Modify: `E:\Data\src\JavaPrj\bazi\src-tauri\src\bazi.rs`

**Interfaces:**
- Consumes: existing `calculate_bazi`, `BaziRequest`, `CalendarType`, `Gender`, `YearEra`.
- Produces: tests that require detailed `ge_ju_basis` and ranking behavior.

- [ ] Update existing `ge_ju_prefers_month_hidden_ten_god_that_appears_in_stems` assertions to require `月支戌藏戊(正官)、辛(偏印)、丁(偏財)`, `辛金透於時干`, and `戊土、丁火未透`.
- [ ] Add direct unit test for a synthetic month branch with multiple transparent candidates where 本氣 wins over lower-layer transparent candidate.
- [ ] Run `cargo test ge_ju -- --nocapture` and verify RED.

### Task 2: Implement candidate scoring and detailed explanation

**Files:**
- Modify: `E:\Data\src\JavaPrj\bazi\src-tauri\src\bazi.rs`

**Interfaces:**
- Consumes: `month_hidden_stems`, `month_branch_ten_gods`, and stem ten-god/name pairs.
- Produces: `analyze_ge_ju(...) -> GeJuResult` with deterministic ranking and richer `basis`.

- [ ] Add small internal structs for hidden candidates and visible stem info.
- [ ] Change `analyze_ge_ju` to receive 月支名/月令藏干/月令藏干十神/天干資訊.
- [ ] Rank transparent candidates by hidden layer first, visible pillar second.
- [ ] Build basis string listing all 月令藏干 pairs, transparent candidates, non-transparent candidates, and chosen reason.
- [ ] Run targeted cargo test and then full cargo test.

### Task 3: Verify app build

**Files:**
- No new files beyond backend changes.

**Interfaces:**
- Produces: passing `npm run build`.

- [ ] Run `npm run build`.
- [ ] Confirm frontend compiles because API fields are unchanged.
