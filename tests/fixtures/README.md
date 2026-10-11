# Classroom UI fixtures

These JSON files are **unaltered `quantModel.lessonContent.threeStepAnalysis` subtrees** of complete production `analyze_pillars` responses, exported by the test-only `LESSON_QA_OUTPUT_DIR` opt-in in `src-tauri/src/bazi.rs`. Other response fields, not analysis fields, are omitted. Exported 2026-09-07 and regenerated 2026-10-10; rule version `classroom-2026-10-v2`.

| File | Four pillars (year / month / day / hour) | Provenance and purpose |
| --- | --- | --- |
| lesson-general.json | 戊申 / 丙辰 / 庚午 / 癸未 | Synthetic structural input; four separate conditional plans |
| lesson-source.json | 甲寅 / 癸酉 / 壬午 / 癸卯 | 2026-09-02 source chart; selected ordinary candidate. Since `classroom-2026-10-v2` the 四比四「得令者為旺」rule gives 強候選, matching the teacher |
| lesson-single.json | 甲寅 / 甲寅 / 甲寅 / 甲寅 | Synthetic structural input; single guarded ordinary fallback, not a dated real-life chart |
| lesson-insufficient.json | 戊戌 / 己丑 / 甲午 / 丁巳 | Synthetic structural input; actual insufficient strength/preferences and no plans |

Tests must assert independently known visible facts and keep competing role cards separate. Intentional text mutations for escaping tests must use cloned data and must not be represented as original Rust outputs. Do not silently hand-edit analytical fields to fit UI expectations.

`lesson-0902-content.json` is the unaltered **complete `quantModel.lessonContent` subtree**, exported 2026-09-09 from `source-2026-09-02.json` using the same Rust QA opt-in. It tests the four exact-source annotation cards together with the existing three-step analysis. The QA exporter uses its existing direct-input request helper; the cards describe the historical male lesson case, not a newly dated luck calculation for the request gender. Regenerate with `LESSON_QA_OUTPUT_DIR` and extract the subtree rather than editing the fixture.
