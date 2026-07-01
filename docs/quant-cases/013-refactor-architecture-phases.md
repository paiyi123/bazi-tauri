# 量化模型架構移植紀錄

這份文件原本記錄 `bazi-sources` 裡 `QuantModelCalculator` 從單一大類別逐步拆成多個 Java service / engine 的過程。移植到 BaZi Desktop 後，實作已收斂到 `src-tauri/src/quant_model.rs`，但「先裁決局勢、再計分」的分層仍然保留。

## 目前 App 版的重構目標

在本機 Rust App 裡，重點不是維持 Java class 拆分，而是讓同一個 Rust 模組內的資料流清楚：

- 輸入整理
- 候選偵測
- 規則裁決
- 天干計分
- 地支計分
- 大運流年計分
- 用神、從格、殺印等輔助分析

核心原則：

- 先定局勢，再計分
- 候選 function 只找候選，不直接改分
- 裁決 function 產生 `ResolvedChartState`
- score function 只吃已裁決 state，不自己重新判三會、三合、半合、六合、相沖

## 目前主要位置

### 協調層

- `build_quant_model_response`
  - 串起：
    1. `build_natal_inputs`
    2. `score_resolved_chart`
    3. `analyze_yong_shen`
    4. `analyze_cong_pattern`
    5. `analyze_sha_yin`
    6. `build_natal_interactions`
    7. `build_luck_scores`

### 輸入與狀態層

- `CalculationInputs`
- `StemInput`
- `BranchInput`
- `ResolvedChartState`
- `ResolvedScoreBundle`

### 偵測與裁決層

- `resolve_chart_state`
- `resolve_three_meeting_candidates`
- `resolve_three_combination_candidates`
- `resolve_half_combination_candidates`
- `resolve_three_meeting_transforms`
- `resolve_three_combination_transforms`
- `resolve_half_combination_transforms`
- `resolve_six_combination_transforms`
- `resolve_six_combination_effects`
- `resolve_branch_prune_effects`
- `resolve_branch_qi_modifiers`
- `resolve_stem_combination_effects`
- `resolve_stem_clash_effects`

### 計分層

- `score_stems`
- `score_branches`
- `score_branch`
- `build_natural_branch_qi_slices`
- `build_transformed_branch_qi_slices`
- `resolve_stem_root`
- `resolve_branch_transparencies`
- `resolve_transformed_branch_transparencies`
- `build_luck_scores`
- `build_annual_score`

### 規則/查表層

- `position_weight`
- `hidden_stems_of`
- `hidden_ratios`
- `resolve_ten_god`
- `resolve_ten_god_score`
- `resolve_branch_element`
- `resolve_*_factor`
- `stem_clash_remainder`
- `branch_clash_score_multiplier`
- `root_multiplier`
- `transparency_multiplier`

### 能量與互動型別

- `BranchTransform`
- `BranchTransformSlice`
- `BranchTransformSliceKind`
- `BranchQiSlice`
- `BranchInteraction`
- `BranchPruneEffect`
- `BranchQiModifier`
- `SixCombinationEffect`
- `StemCombinationEffect`
- `StemClashEffect`
- `RootApplication`
- `TransparencyApplication`

### 候選/組合定義型別

- `HalfCombinationCandidate`
- `ThreeCombinationCandidate`
- `ThreeMeetingCandidate`
- `BranchTransformCategory`
- `CombinationBranchRole`
- `BranchClashType`

## 舊 Java 分層在 Rust 版的對應

| 舊 Java 名稱 | Rust App 版對應 |
| --- | --- |
| `QuantModelCalculator` | `build_quant_model_response`、`score_resolved_chart` |
| `QuantModelInteractionDetector` | `resolve_*_candidates` |
| `QuantModelInteractionResolver` | `resolve_chart_state` 與 `resolve_*_transforms` |
| `QuantModelTransformResolverSupport` | `create_branch_transform`、`merge_transforms`、`can_participate_in_second_transform` |
| `QuantModelEffectResolverSupport` | `resolve_six_combination_effects`、`resolve_branch_prune_effects`、`resolve_branch_qi_modifiers` |
| `QuantModelStemScoreEngine` | `score_stems`、`resolve_stem_root` |
| `QuantModelBranchScoreEngine` | `score_branches`、`score_branch`、`build_*_branch_qi_slices` |
| `QuantModelLuckScoreEngine` | `build_luck_scores`、`build_annual_score` |
| `QuantModelRuleTables` / `LookupSupport` | `position_weight`、`hidden_stems_of`、`hidden_ratios`、`resolve_ten_god_score` |

## 新規則建議落點

之後若要補《量化五行力》新規則，建議依性質放置：

- 新的候選規則：
  - 放在 `resolve_*_candidates` 附近。
- 新的優先級/壓制規則：
  - 放在 `has_stronger_*`、`should_suppress_*`、`can_participate_in_second_transform` 這類 helper 附近。
- 新的 transform、主化、次化、餘氣規則：
  - 放在 `create_branch_transform`、`resolve_*_factor`、`resolve_*_residual_factor` 附近。
- 新的沖後剪枝、餘力、殘量規則：
  - 放在 `resolve_branch_prune_effects`、`resolve_branch_qi_modifiers`、`build_branch_clash_qi_modifier` 附近。
- 新的固定表、倍率、對照表：
  - 放在檔案後半段的查表 helper。
- 天干最終列分：
  - 放在 `score_stems` 或 `resolve_stem_root`。
- 地支最終列分：
  - 放在 `score_branch`、`build_natural_branch_qi_slices`、`build_transformed_branch_qi_slices`。
- 大運流年分數：
  - 放在 `build_luck_scores`、`build_annual_score`、`build_extended_inputs`。

## 目前仍可繼續優化的方向

- 把 `quant_model.rs` 依功能拆成多個 Rust module，降低單檔維護成本。
- 把更多案例筆記轉成 `cargo test` 可跑的回歸測試。
- 把第五篇 `主化 + 次化 + 餘氣` 的數值規則完全改由 slice 主導。
- 補一份「規則對照表」，把書本章節、docs case、Rust function、UI 欄位串起來。
- 針對大運與流年補更多「原局 + 大運 + 流年共同裁決」的測試。

## 一句話總結

BaZi Desktop 版目前是：

- 由 `bazi.rs` 建立四柱與歲運
- 由 `quant_model.rs` 先 resolve 出裁決後局勢
- 再由天干、地支、歲運計分 function 輸出結果
- 最後讓 Vue UI 只顯示 `QuantModelResponse`

後續擴規則時，應延續「先裁決、再計分」的資料流，而不是在 UI 或單一得分步驟臨時補判斷。
