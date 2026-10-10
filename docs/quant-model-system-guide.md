# 量化五行力系統說明

這份文件整理 BaZi Desktop 目前內建的「量化五行力」功能。它是從原本 `bazi-sources` 的服務端模型移植、收斂到本機 App 後的工程導覽，重點是讓後續維護者能快速找到資料流、規則落點、UI 對應與案例筆記。

重要前提：

- 本專案是依《量化五行力》重建出的工程版本，不是逐頁逐字的完整數位化。
- 目前量化模型在本機 Rust 端執行，不依賴 Spring Boot、外部 API 或資料庫。
- 文件會刻意區分三件事：
  - 書上較穩定的規則底盤
  - 目前 App 已實作的部分
  - 仍屬工程化近似、待補案例與待補測試的部分
- 書中篇章對應，請以 `docs/quant-cases/README.md` 的整理為準。

## 1. App 內的主要架構

### 1.1 整體資料流

1. Vue 表單送出出生資料或直接輸入四柱。
2. `src/services/bazi.ts` 透過 Tauri `invoke` 呼叫 Rust command：
   - `calculate_bazi`
   - `analyze_pillars`
3. Rust 入口在 `src-tauri/src/bazi.rs`：
   - `calculate_bazi`：由公曆 / 農曆輸入推四柱、大運、流年。
   - `analyze_pillars`：直接以四柱建立分析資料，必要時才推候選西元年。
4. `bazi.rs` 整理出日主、四柱、藏干、大運資料後，呼叫 `src-tauri/src/quant_model.rs` 的 `build_quant_model_response`。
5. `quant_model.rs` 產生 `QuantModelResponse`，包含：
   - 總分與強弱判定
   - 天干列分
   - 地支列分
   - 原局合沖互動
   - 大運與流年量化
   - 用神、從格、殺印相生等輔助分析
6. 前端 TypeScript 型別在 `src/types/bazi.ts`，UI 元件只負責顯示 Rust 已算好的資料。

開發模式下，如果只跑 `npm run dev` 而不在 Tauri runtime 裡，`src/services/bazi.ts` 會改用 `src/debug/browser-fixtures.ts` 的 fixture，方便看版面；正式 App 與 `npm run tauri:dev` 則走 Rust invoke。

### 1.2 Rust 量化核心分層

目前模型集中在 `src-tauri/src/quant_model.rs`。原本 Java 版的多個 service / engine，在本 App 裡多數已收斂成同一個 Rust 模組中的 private struct 與 function。

| 層級 | 主要 Rust 位置 | 職責 |
| --- | --- | --- |
| 入口協調 | `build_quant_model_response` | 組裝整個量化流程，回傳 `QuantModelResponse` |
| 輸入標準化 | `build_natal_inputs`、`CalculationInputs`、`StemInput`、`BranchInput` | 把四柱資料轉成統一輸入格式 |
| 局勢裁決 | `resolve_chart_state` | 先找候選，再產生裁決後的 `ResolvedChartState` |
| 三會 / 三合 / 半合候選 | `resolve_three_meeting_candidates`、`resolve_three_combination_candidates`、`resolve_half_combination_candidates` | 找出可能成立的地支互動 |
| 化局與餘力 | `resolve_*_transforms`、`create_branch_transform`、`merge_transforms` | 決定哪些支進入化局、保留多少餘氣 |
| 合沖效果 | `resolve_six_combination_effects`、`resolve_branch_prune_effects`、`resolve_branch_qi_modifiers`、`resolve_stem_*_effects` | 處理六合不化、沖後剪枝、天干合沖 |
| 天干計分 | `score_stems`、`resolve_stem_root` | 計算可見天干的本分、位調、合沖餘量與得根 |
| 地支計分 | `score_branches`、`score_branch`、`build_*_branch_qi_slices`、`resolve_branch_transparencies` | 計算藏干、化後切片、透干增值、祿根與多重支互動 |
| 大運流年 | `build_luck_scores`、`build_annual_scores`、`build_extended_inputs` | 把大運、流年加入同一套裁決與計分管線 |
| 輔助分析 | `analyze_yong_shen`、`analyze_cong_pattern`、`analyze_sha_yin` | 產出用神、從格、殺印相生說明 |
| 規則查表 | `position_weight`、`hidden_stems_of`、`hidden_ratios`、`resolve_ten_god_score`、`root_multiplier`、`transparency_multiplier` | 十神分值、位置權重、藏干比例、根氣倍率、合沖倍率 |

### 1.3 前端顯示分層

前端目前是展示層，不自行判命理規則：

- `src/components/QuantFivePhasePanel.vue`
  - 量化結果總面板
- `src/components/QuantStemScoreTable.vue`
  - 顯示天干列分
- `src/components/QuantBranchScoreList.vue`
  - 顯示地支列分與細節
- `src/components/LuckQuantTable.vue`
  - 顯示大運與流年量化結果
- `src/components/FiveElementStrengthPanel.vue`
  - 顯示用神、從格、殺印相生等輔助分析
- `src/components/PillarPanel.vue`
  - 在四柱視圖整合原局、大運、流年的互動表
- `src/components/FiveElementFlowPanel.vue`
  - 依 `QuantModelInteraction` 畫出五行與合沖流向
- `src/composables/useLuckDetailPopover.ts`
  - 處理大運流年細節彈窗

前端的責任是把 Rust 回傳的 `summary / stemScores / branchScores / interactions / luckScores` 做成可讀 UI。若 UI 顯示與模型結果不一致，優先檢查 `src/types/bazi.ts` 與元件欄位對應。

## 2. 《量化五行力》各篇的計算處理對應

### 2.1 章節對應總覽

| 篇次 | 主題 | 目前落地位置 | 現況 |
| --- | --- | --- | --- |
| 第 1-2 篇 | 基礎分值、十神與位置底盤 | `position_weight`、`hidden_stems_of`、`hidden_ratios`、`resolve_ten_god_score`、`score_stems`、`score_branches` | 已作為整個模型底盤 |
| 第 3 篇 | 用神 | `analyze_yong_shen`、`docs/quant-cases/016-book-part3-yongshen-notes.md` | 已有第一版工程實作 |
| 第 4 篇 | 沖合論 | `resolve_chart_state` 與相關 `resolve_*` functions、`docs/quant-cases/005...`、`017...` | 已實作核心優先級與多個關鍵例，未完全覆蓋全書時序 |
| 第 5 篇 | 合化力量量化 | `create_branch_transform`、`resolve_*_factor`、`score_branch`、`docs/quant-cases/009...`、`018...` | 已落地主要倍率與多合局處理 |
| 第 6 篇 | 從格 / 不從 | `analyze_cong_pattern`、`docs/quant-cases/019...` | 已有門檻式工程版 |
| 第 7 篇 | 精算案例 | `score_branch`、`resolve_branch_qi_modifiers`、`docs/quant-cases/020...` | 以案例驅動方式部分落地 |
| 第 8 篇 | 大運流年 | `build_luck_scores`、`build_annual_score`、`docs/quant-cases/014...`、`015...` | 已做成共享裁決管線 |
| 第 9 篇 | 殺印相生 | `analyze_sha_yin`、`docs/quant-cases/021...` | 已有工程化判讀 |
| 第 10 篇 | 進階從格 | `analyze_cong_pattern`、化局裁決 functions、`docs/quant-cases/022...` | 部分門檻已吸收，仍需更多案例 |
| 第 11 篇 | 干支義象理論 | `docs/quant-cases/023...` | 目前主要作為理論依據，未直接新增獨立算分規則 |
| 第 12 篇 | 名人時辰比對 | 直接輸入四柱模式、`analyze_pillars`、`docs/quant-cases/024...` | App 可逐一比較候選時辰，尚未提供批次候選工具 |
| 第 13 篇 | 特例、真太陽時、殘留難題 | `docs/quant-cases/025...` | 以人工輔助分析方式處理，未完全自動化 |

### 2.2 目前工程版最完整的部分

目前落地最完整、最穩定的是這幾塊：

- 基礎十神分值與位置權重
- 四柱地支藏干拆分
- 天干得根與地支透干增值
- 三會、三合、半合、六合的候選偵測與優先級裁決
- 六合不化、天干相沖、地支相沖後的餘力與剪枝
- 大運與流年加入同一裁決管線
- 量化結果與 UI 面板的資料結構

相對來說，仍較依賴案例筆記與工程近似的部分有：

- 高階多局互制的完整通用化
- 某些後半書精算特例
- 真從 / 假從邊界
- 候選時辰批次比較與人生驗證型命例
- 更細的 Rust 單元測試與回歸測試

## 3. 年月日時干支的五行強弱計算

### 3.1 輸入模型

目前量化模型對四柱的處理方式是：

- 天干實際列分欄位只有 `年干`、`月干`、`時干`
- `日干` 是日主，只作為比較基準，不另列一筆分數
- 地支列分欄位有 `年支`、`月支`、`日支`、`時支`

這個設計可以在 `build_natal_inputs` 看到：

- 天干索引：年 `0`、月 `1`、時 `3`
- 地支索引：年 `0`、月 `1`、日 `2`、時 `3`

索引不是只為了排序，也用來算距離，決定「緊貼 / 隔一位 / 隔二位」的權重與衝擊強度。

### 3.2 十神本分

目前穩定採用的十神基礎分值如下，由 `resolve_ten_god_score` 提供：

- 比肩、劫財：`+100`
- 正印、偏印：`+80`
- 食神、傷官：`-45`
- 正財、偏財：`-60`
- 正官、七殺：`-75`

這是所有後續位調、得根、透干、合沖修正之前的底盤。

### 3.3 位置權重

`position_weight` 目前採用：

- 年干：`45`
- 月干：`55`
- 時干：`50`
- 年支：`45`
- 月支：`100`
- 日支：`55`
- 時支：`50`
- 運干：`35`
- 運支：`25`
- 流年干：`20`
- 流年支：`20`

也就是：

- 月令最大
- 月干與日支次高
- 時柱中段
- 年柱較遠
- 大運與流年能入局，但不直接壓過原局核心位置

### 3.4 藏干比例

地支先拆成藏干，再按藏干比例換算：

- 一藏干：`1.00`
- 二藏干：`0.63 / 0.37`
- 三藏干：`0.50 / 0.25 / 0.25`

這些比例由 `hidden_ratios` 提供。

### 3.5 天干的計分流程

`score_stems` 的順序可以概括成：

1. 決定這個天干的有效五行。
2. 若有天干合化，先看是否轉成化後五行。
3. 依日主算十神本分。
4. 乘上位置權重，得到位調後分數。
5. 若有天干合或沖，再乘上剩餘倍率。
6. 掃描所有地支，找這個天干是否得根。
7. 依祿根 / 中根 / 微根 / 墓根與距離，加上得根 bonus。

可以用近似公式表示為：

`天干最終分 = 位調後本分 × 干合干沖餘量 + 所有得根增值`

得根增值不是固定常數，而是同時看：

- 根級別倍率
- 與該地支距離
- 該支是否已被化局、剪枝、沖損
- 該支在六合不化後是否只剩餘力

### 3.6 地支的計分流程

`score_branches` / `score_branch` 比天干複雜，因為它要同時處理：

- 自然藏干拆分
- 合化後的化神切片
- 沖後只留主氣或旁氣折減
- 支內各分項被透干放大
- 祿根整支改寫
- 多重合局的不化提升

其主流程是：

1. 先讀 `ResolvedChartState`，不再自行判局。
2. 若該支被沖後剪枝，只保留剩餘主氣與比例。
3. 若該支已化局，改用 `BranchTransformSlice` 計分。
4. 若尚未化局，按原藏干比例逐項計分。
5. 對支內每個分項計算十神本分、比例、位置權重、合沖倍率。
6. 再看該支之氣是否透到天干，追加 transparency bonus。
7. 若該支是日主祿地，直接以 `+100` 改寫整支底盤，再疊透干。
8. 若遇三會不化、雙三合不化等特例，再補額外增值。

可近似理解成：

`地支最終分 = 支內各切片位調後總和 + 透干/特例增值`

### 3.7 年月日時在強弱判斷中的角色

四柱不是平均出力，而是按位置與互動共同決定：

- 月支最重，所以月令常主導整體傾向
- 年支與時支雖較遠，但若成局、得根、透干，仍可大幅拉動總分
- 日支雖不是日主本人，但因位置接近日主，對強弱判定很重要
- 日干本身不列分，避免把日主同時當裁判又當選手重複加權

## 4. 合與沖的判斷

### 4.1 兩段式思路：先偵測，再裁決

這個專案不讓 score function 直接各自判合沖，而是採兩段式：

1. `resolve_*_candidates`
   - 只找出候選，不決定誰成立。
2. `resolve_chart_state`
   - 依優先級與占位情況決定最終生效局。

這樣做的好處是：

- 避免計分時重複判斷
- 同一套裁決結果可同時供天干、地支、大運、流年使用
- `details`、`combineNote`、`interactions` 可以直接回溯最終裁決

### 4.2 地支合化的優先順序

目前 resolver 採用的主順序是：

`三會 > 三合 > 半合 > 六合`

實作上對應到：

- 先解三會
- 再補沒有被占走的三合
- 再補半合
- 最後才看六合

核心原則是「先占支位者優先」。

### 4.3 合化與不化分開處理

程式把「化局成立」與「有互動但不化」分成不同層：

- `BranchTransform`
  - 表示該支真的進入化局，後續按化後五行列分。
- `SixCombinationEffect`
  - 表示六合而不化，只保留餘力，不改五行本質。

所以現在不是單純一句「有合就改成化神」，而是會判斷：

- 是否足以化
- 化後是主化還是次化
- 是否仍保留餘氣
- 若不化，雙方還剩多少原力

### 4.4 沖的分類

`classify_branch_clash` 目前把地支沖分成：

- 專氣沖：`子午`、`卯酉`
- 長生沖：`寅申`、`巳亥`
- 墓庫沖：其他墓庫型沖，如 `辰戌`

不同類型的後續處理不同：

- 專氣沖
  - 近沖時整支可直接化零。
  - 隔一位、隔二位保留不同餘量。
- 長生沖
  - 主要處理受沖支氣折減。
- 墓庫沖
  - 常表現為旁氣化零，只保留主氣。

### 4.5 距離與倍率

目前程式對距離的工程化處理很明確：

- 天干緊貼相沖：`×0.5`
- 天干隔一位相沖：`×0.875`
- 專氣支近沖：`×0`
- 專氣支隔一位沖：`×0.75`
- 專氣支隔二位沖：`×0.89`
- 六合不化餘力：`×0.52`

主要落點：

- `stem_clash_remainder`
- `branch_clash_score_multiplier`
- `branch_clash_affected_multiplier`
- `resolve_six_combination_effects`

## 5. 複雜多合沖情況處理

這部分是目前工程版最花力氣的地方，也是最需要測試保護的地方。

### 5.1 同一支可能同時牽涉多個局

多合沖處理的核心不是「把所有規則依序硬套」，而是先處理支位競爭。

例如同一支可能同時參與：

- 半合
- 六合
- 三合
- 三會
- 夾沖

目前作法是：

- 強局先占位
- 弱局只能在剩餘支位上成立
- 某些特殊支位才允許二次化

### 5.2 可二次參與合化的特殊支

`can_participate_in_second_transform` 目前只允許四專氣支在特定條件下再接第二個化局：

- `子`
- `午`
- `卯`
- `酉`

原因是書中有些案例顯示，這四支在合化前後性質較穩，可做「先化、再接同元素六合」。

### 5.3 已落地的多局特例

目前 code 中有明確處理的情況包括：

- 左右六合並見時，以距離與位置權重選擇成立的一側
- 同元素雙六合夾住中支時，中支只剩 `0.04`
- 左右夾沖專氣支時，兩側外支可改成 `×0.5`
- 半合先成立後，若同元素六合仍合理，可標記為可再接六合
- 三會不化時，仍可把專氣支提升到目標 element units
- 雙三合不化時，仍可給目標支額外增值
- `二寅夾沖申` 類型的支氣折減

### 5.4 複雜局的工程守則

專案目前遵守幾個重要守則：

- 化局先決定「這支算什麼」
- 剪枝與餘力再決定「還剩多少」
- 得根與透干一定讀裁決後狀態，不回頭讀原始四柱
- 不讓同一份力量在原藏干、化神、得根、透干重複高估

這也是 `docs/quant-cases/026-implementation-guidelines-avoid-overcounting.md` 的重點。

### 5.5 目前仍保留為待補的複雜情況

雖然已補了不少多局互動，但仍有部分案例未完全通用化。例如文件中提到的「寅亥合緩和巳亥沖的 0.64 分攤算法」仍比較像待校準規則，不宜在 UI 或文件中宣稱已完整解完。

目前專案對多合沖的策略是：

- 先把穩定案例做成明確規則
- 對仍不穩定的書本精算法，先保留案例與筆記
- 補實作時同步補 Rust 測試，避免規則漂移

## 6. 大運流年加入的五行處理

### 6.1 基本做法

`build_luck_scores` 的核心理念是：

- 不另外寫一套「運勢版規則」
- 而是把大運、流年當成額外柱位，送進同一套裁決與計分管線

也就是：

- 大運：`原局四柱 + 大運整柱`
- 流年：`原局四柱 + 大運整柱 + 流年一柱`

### 6.2 大運與流年的權重

`position_weight` 額外定義：

- 運干：`35`
- 運支：`25`
- 流年干：`20`
- 流年支：`20`

這讓運勢柱位能參與計分，但不直接壓過原局四柱核心位置。

### 6.3 大運的處理方式

大運列分不是把干、支各自孤立計算，而是：

1. 把大運整柱附加到原局。
2. 先做一次完整合沖裁決。
3. 再從結果中抽出 `運干` 與 `運支` 的分數。
4. UI 仍用「前五年看運干、後五年看運支」顯示。

也就是說，顯示雖分前後半運，但分數來源仍是「大運整柱已入局後」的結果。

### 6.4 流年的處理方式

流年比大運再多一層：

1. 原局四柱先在場。
2. 大運整柱先入場。
3. 流年整柱再入場。
4. 三者一起走同一輪合沖裁決。
5. 最後抽出 `流年干`、`流年支`、`運干`、`運支`。

這代表流年分數不是孤立分數，而是「受原局與大運共同作用後的分數」。

### 6.5 R 值、作用後命局、傾向

`ratio` 與 `tendency_label` 這一層不重新算命理，只做結果轉譯：

- `impactRatio`
  - 用 `luckScore / sqrt(natalScore^2 + luckScore^2)` 正規化。
- `effectiveNatalScore`
  - 以 `natalTotal + luckTotal` 顯示運程作用後的命局強弱。
- `tendency`
  - 判斷是「較利平衡」還是「較加重失衡」。

前端大運表與流年展開表都直接吃這組結果。

### 6.6 為什麼要用共享裁決管線

這樣設計有三個實際好處：

- 大運與流年真的會改變原局內部合沖，而不只是額外加減分
- 同一個書本案例可在本命、歲運場景下共用規則
- 不容易出現「本命一套、運勢一套」導致的規則分叉

## 7. 測試與案例筆記

### 7.1 目前測試狀態

目前 repo 內的 Rust 測試主要集中在 `src-tauri/src/bazi.rs` 的 `mod tests`，已覆蓋：

- Android response 需要保留完整大運與量化資料
- 出生資料排盤的基本資料結構
- 直接輸入四柱與候選年份等輔助流程

量化模型本身的大量細節仍主要保存在 `docs/quant-cases/`。也就是說，案例筆記已經很豐富，但還有不少內容尚未轉成 Rust 單元測試。這是後續維護量化規則時最值得補強的地方。

### 7.2 案例筆記與測試的關係

`docs/quant-cases/` 的角色很重要，它不是純備忘錄，而是測試設計的來源庫。

典型流程是：

1. 先把書本案例或差距案例寫成筆記。
2. 標記目前 App 與書本相符或不符的地方。
3. 再把穩定結論轉成 Rust 測試。

所以這個資料夾同時承擔：

- 書本篇章映射
- regression case 記憶
- 已知 gap 記錄
- 後續 refactor 的 guardrail

### 7.3 後續測試特別應重視什麼

量化測試不要只看單一總分，應優先鎖住這些可回歸的細節：

- `combineNote` 是否符合預期
- `details` 是否有出現正確的互動說明
- 某柱分數是否被正確化零、剪枝、保留餘氣
- 多局並見時哪一側成立
- 大運與流年是否真的進入共同裁決
- 用神、從格、殺印結論是否因總分或主導家族改變而合理變動

這種測法的好處是，重構時比較容易知道是：

- 規則順序變了
- 倍率變了
- 或只是說明字串漏了

### 7.4 基準案例與已知差距

目前文件中有兩類重要案例：

- benchmark case
  - 用來守住已經穩定的書本或工程結果。
- known gap case
  - 用來承認目前尚有落差，但先把差距固定下來。

例如：

- `001-1970-03-08-ding-hai.md`
  - 適合當基礎負分 benchmark。
- `002-jane-zhang-gap-case.md`
  - 適合保留為祿根、透干或進階修正的 known gap。

### 7.5 建議執行命令

修改 Rust 計算邏輯後至少跑：

```bash
npm run build
```

以及：

```bash
cd src-tauri
cargo test
```

若修改 UI 顯示，再用：

```bash
npm run tauri:dev
```

在 App 內確認「四柱」「運勢」「量化」「分析」幾個面板是否都正常。

## 8. 目前應如何閱讀這個專案

如果是第一次進來，建議閱讀順序如下：

1. 先看 `src-tauri/src/bazi.rs` 的 `calculate_bazi` 與 `analyze_pillars`。
2. 再看 `src-tauri/src/quant_model.rs` 的 `build_quant_model_response`。
3. 接著看 `resolve_chart_state`，理解裁決後狀態如何建立。
4. 再分別看 `score_stems` 與 `score_branches`。
5. 若要看歲運，再看 `build_luck_scores` 與 `build_annual_score`。
6. 若要核對 UI，對照 `src/types/bazi.ts` 與 `src/components/QuantFivePhasePanel.vue`。
7. 若要核對書本章節，再配合 `docs/quant-cases/README.md`。

## 9. 目前邊界與未完成項

目前這個工程版最適合做的是：

- 四柱五行強弱的量化拆解
- 合沖先後與局力的工程化重建
- 大運流年共同裁決
- 書本案例回歸與 UI 對照
- 本機離線 App 內的快速分析

仍應保守看待的部分是：

- 全書所有後半段精算特例已完整覆蓋
- 所有真從 / 假從邊界都已定論
- 所有多合沖都已通用化
- 候選時辰能自動選出唯一正確結果
- 所有人生驗證型命例都能只靠四柱自動判完

因此這份專案比較正確的定位是：

- 已有穩定核心
- 已能支撐持續擴充
- 但仍需要靠 `docs/quant-cases` 與 Rust 測試，逐篇把書中的細節補完整
