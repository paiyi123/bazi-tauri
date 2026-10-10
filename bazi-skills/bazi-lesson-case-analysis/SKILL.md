---
name: bazi-lesson-case-analysis
description: Use when organizing BaZi class recordings, whiteboard notes, or lesson cases into an explainable analysis of 格局、用神、喜忌、合沖會局與大運. Separate observed chart facts, teacher-specific rules, evidence, uncertainty, and conclusions; use when adding or reviewing lesson rules for this project.
---

# BaZi Lesson Case Analysis

Turn a lesson into traceable rules, never unsupported fortune-telling. Read the relevant dated references, including `references/2026-07-01-lesson.md`, `references/2026-07-08-lesson.md`, `references/2026-07-15-lesson.md`, `references/2026-07-22-lesson.md`, `references/2026-07-29-lesson.md`, `references/2026-08-05-lesson.md`, and `references/2026-08-12-lesson.md`, before adding a rule to the App.

## Workflow

1. Extract only verified four pillars, month command, hidden stems, ten gods, and interactions. Mark unclear transcript or whiteboard text as `待核對`.
2. State the teacher rule separately from chart facts. Label it `本課派規則`; do not present it as a universal rule.
3. Explain the evidence chain in this order: 定格 → 取用神 → 明喜忌 → 大運相對比較.
4. For each conclusion, give its supporting pillar/interaction and rejected alternatives. Preserve the App's normal `geJu` as the reference when a special pattern is considered.
5. Use comparative language for luck: `較前運穩定` does not mean `必然吉`.
6. Translate a rule into data before code: identify inputs, decision gates, output label, evidence, and fallback. Keep `待核對` or `尚未接入` in the dated reference documents rather than silently treating a classroom hypothesis as production logic; the App only receives rules or exact cases intentionally selected for inference display.

## Hard Gates

- If any pillar, day stem, month command, or source transcription is uncertain or conflicting, output only a `待核對` summary in the dated reference document. Set `verification=image_uncertain` (or the matching source state), `algorithm_status=本課派規則・待四柱確認`, and `fixtureAllowed=false`; do not add it to App inference unless the product requirement explicitly asks to display blocked diagnostics.
- Never choose one character from conflicting ASR/photo candidates. Preserve the alternatives and source location; do not create a fixture, production score, or use-god/avoid-god conclusion from that uncertainty.
- A rule may move from `candidate` to `verified` only with a computable predicate, stated threshold/source, one passing fixture, one rejection or break-pattern fixture, and a regression test. Otherwise it remains documentation.
- Only `verified` rules with passing fixtures may become `implemented` automatic inference. The normal `geJu` remains visible until then.

Red flags: deadline pressure, teacher authority, “看起來像”四柱, invented numeric thresholds, or a request to add health/death/guaranteed-event output. Stop at `待核對` and mark sensitive claims `不納入自動推論`.

## Algorithm Mapping

For a candidate special pattern, use this order:

`資料核對 → 月令/旺勢 → 一行是否獨旺 → 破格控制元素 → 定格 → 用神/喜忌 → 證據鏈`

The existing quantitative model remains the source of computed facts. Lesson rules annotate or compare those facts; they do not overwrite the normal `geJu` result until the rule has verified fixtures and regression tests.

## Lesson Inference Contract

Treat source verification and algorithm maturity as separate axes:

- `sourceVerification` records whether the transcript and whiteboard agree.
- `algorithmStatus` is `candidate`, `verified`, `implemented`, or `blocked`.
- An exact lesson chart uses `fixtureScope=source_case_only`; it proves the annotation lookup, not a generalized strength/use-god rule.
- `outputMode=annotation_only` and `fallbackPolicy=preserve_normal_outputs` keep `geJu`, quantitative strength, use gods, and special-pattern outputs unchanged.
- `appReference` and `lessonCandidate` are shown side by side. Record `agreementStatus` rather than forcing agreement.
- Include a one-pillar near-match negative fixture for every exact-chart lesson rule.
- For a generalized day-master state rule, keep source examples and the App predicate separate: a transcript phrase such as 「得勢多」 is not a numeric threshold. Any engineering threshold remains `candidate`, labels any suggested elements as directions rather than formal 用神, and never overwrites normal outputs.
- For 2026/07/29 interaction examples, reuse the existing `QuantModelInteraction` evidence for 半合、三合 and 沖; do not invent a second scoring path. An exact source chart may be `source_case_only`, while unreadable whiteboard charts remain `image_uncertain` with `fixtureAllowed=false`.
- For 2026/08/05, map 扶抑 only as a `candidate` comparison to the existing `QuantYongShen`; it must not select or overwrite App 用神. 病藥 lacks a generalizable priority gate and computable threshold, so it remains documentation until it has explicit positive and negative fixtures. Keep incomplete whiteboard charts as `image_uncertain`.
- For 2026/08/12, only the exact `癸卯 乙卯 辛亥 庚寅`, `己卯 甲戌 乙未 庚辰`, and transcript-exact `壬戌 己酉 丁丑 甲辰` charts may become `source_case_only` annotations, each with a one-pillar rejection fixture. The first textbook chart remains `transcript_uncertain`. Do not generalize the exact 從財 candidate, 病藥 mapping, or 「旺神沖衰、衰神沖旺」 until their predicates, thresholds, break conditions, and positive/negative fixtures are explicit. Reuse `QuantModelInteraction` evidence and preserve all normal outputs.

## Required Output

| Section | Required content |
| --- | --- |
| 事實 | Four pillars and directly computed interactions only |
| 規則 | The exact 本課派規則 and its scope |
| 證據 | Pillar, hidden stem, combination/clash, or score supporting the rule |
| 結論 | Pattern, use gods, avoid gods, or relative luck result |
| 不確定性 | Missing birth data, unreadable text, or rules not yet implemented |

## Safety Boundary

Do not infer medical diagnoses, death, self-harm, criminal conduct, or definitive real-world outcomes. Preserve those statements only as a classroom-source note when necessary, and mark them `不納入自動推論`.

## Example

`事實：巳亥沖，亥藏壬甲。規則：依本課派規則，該沖使壬甲根氣需降權。證據：沖支與藏干位置。結論：是否改判從格仍須同時通過全局火土優勢與日主根氣檢查。`
