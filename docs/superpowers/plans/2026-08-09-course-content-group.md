# Course Content Group Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a project-local BaZi lesson-analysis skill and an `上課內容` group between `用神分析` and `量化五行力（書表對照版）`.

**Architecture:** Return a small, structured `lessonContent` payload from the Rust quant-model response. Keep it separate from the existing `geJu`, `yongShen`, and `specialPattern` calculations, so future lesson-specific inference rules add evidence rows rather than changing established results. Render it through a dedicated Vue card and preserve the same relative ordering in compact mode.

**Tech Stack:** Rust/Tauri, serde, Vue 3, TypeScript, Element Plus.

## Global Constraints

- Preserve the existing meaning of `geJu` and the existing normal/special-pattern analysis flow.
- Do not alter the user-owned `src-tauri/Cargo.toml` change or existing untracked files.
- Keep the initial lesson payload educational and explicit that individual teacher rules will be added incrementally.
- Add Rust regression coverage before production code and verify frontend type checking/build.

---

### Task 1: Add and verify the project-local skill

**Files:**
- Create: `bazi-skills/bazi-lesson-case-analysis/SKILL.md`

**Produces:** A concise skill that requires fact/rule/evidence separation and marks classroom-specific rules as such.

- [ ] Initialize `bazi-skills/bazi-lesson-case-analysis` using the supplied skill initializer.
- [ ] Write `SKILL.md` with trigger metadata, required analysis output fields, a safe-boundary rule, and one concrete evidence-chain example.
- [ ] Run `quick_validate.py bazi-skills/bazi-lesson-case-analysis` and inspect the skill against a lesson-case prompt.

### Task 2: Add the lesson-content response contract

**Files:**
- Modify: `src-tauri/src/bazi.rs:343-429`
- Modify: `src-tauri/src/quant_model.rs:322-396`
- Modify: `src/types/bazi.ts:268-381`
- Test: `src-tauri/src/bazi.rs:948-1316`

**Consumes:** Existing `QuantModelResponse` serialization.

**Produces:** `QuantLessonContent` / `lessonContent`, containing title, status, lesson flow, and a future-rule notice.

- [ ] Write a failing Rust test that obtains a direct-pillar response and asserts `lesson_content` exists with the required course title, status, and ordered process.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml course_content` and confirm compilation fails because the response field is absent.
- [ ] Define the serializable Rust DTO, return it from `build_quant_model_response`, and mirror the camelCase TypeScript interface.
- [ ] Re-run the focused Rust test and confirm it passes.

### Task 3: Render the group at the requested location

**Files:**
- Create: `src/components/CourseContentPanel.vue`
- Modify: `src/App.vue:56-108,149-178`

**Consumes:** `BaziResponse.quantModel.lessonContent`.

**Produces:** An `上課內容` card after `FiveElementStrengthPanel` and before `QuantFivePhasePanel`; compact navigation places the analysis tab before quantification and renders the same card after 用神分析.

- [ ] Create the card with an empty-state for responses without `lessonContent`, a lesson status, ordered process, and future-rule notice.
- [ ] Import and place the card in desktop and compact layouts at the specified relative location.
- [ ] Run `npm run build` and correct template/type errors.

### Task 4: Verify integration and review scope

**Files:**
- Verify: `src-tauri/src/bazi.rs`, `src-tauri/src/quant_model.rs`, `src/types/bazi.ts`, `src/components/CourseContentPanel.vue`, `src/App.vue`, `bazi-skills/bazi-lesson-case-analysis/SKILL.md`

- [ ] Run the focused course-content test and the full Rust test suite.
- [ ] Run `npm run build`.
- [ ] Review the diff to ensure the new group does not alter existing 用神、格局或量化結果 and no unrelated dirty files were changed.
