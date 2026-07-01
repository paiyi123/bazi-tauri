# 避免高估的實作準則

## 背景
- 第十篇的 `K先生`、`W先生` 是非常好的校準盤。
- 書上這兩盤大約是：
  - `K先生`：`+292`
  - `W先生`：`+282`
- 目前程式若被後續規則推高到：
  - `K先生`：`+404.9`
  - `W先生`：`+1017.6`
- 問題通常不在「從格判線」，而在：
  - 化局成立後
  - 地支分數先被放大一次
  - 天干得根再從同一股氣吃一次
  - 透干又從同一股氣再吃一次

一句話：
- **成局沒錯，但同一股氣被重複算了兩到三次。**

---

## 核心原則

### 原則 1：合局/化局是「重分配」，不是「原氣 + 新氣」雙算
- 書上的寫法多為：
  - `酉化成 1 個金`
  - `辰化成 0.8 個金`
  - `尚有餘氣 0.2`
- 這表示：
  - 原支氣已被重新分配
  - 不應再保留原支 full strength 後又疊加 transformed strength

工程含義：
- `BranchTransformSlice` 的 `PRIMARY / SECONDARY / RESIDUAL`
  - 應是最終可用氣量拆分
  - 不是額外 bonus layer

### 原則 2：地支計分、天干得根、透干不能同時吃 full transformed energy
- 書上允許：
  - 先用合局/化局結果重算地支列分
  - 再讓天干受益
- 但不表示：
  - 天干可以再把整個 transformed branch 當作完整新根

工程含義：
- 天干得根與透干應吃：
  - `可用增量`
  - 或 `切片倍率`
- 而不是再把整個 transformed branch 視為一個新的完整 root source

### 原則 3：同一合局對同一可見天干，應有「去重」或「封頂」
- 一個天干不能因同一個三會/三合局的多個支，同時得到接近滿額的重複加成。
- 特別是：
  - `W先生` 的 `申酉戌三會化金`
  - `庚辛` 不應從 `申/酉/戌` 三支都吃滿額

工程含義：
- 同一 `transform group`
  - 對同一 visible stem
  - 最多取一次主根
  - 其餘只能降為弱根、增量根，或直接不再重複取用

### 原則 4：局是否成立，與成立後可放大到何種程度，要分開判
- 書上常見：
  - 承認 `三會成立`
  - 但後續加成不會無限擴張
- 因此：
  - `resolve` 階段判定成局
  - `score` 階段仍要控制放大量

### 原則 5：第十篇這類章節案例要做成完整回歸，不是特例硬編碼
- 目的不是寫：
  - `if (K先生) ...`
- 而是把：
  - 總分
  - 關鍵柱分
  - combineNote
  鎖住，當作校準護欄

---

## 建議的實作規則

### 規則 A：化局後的天干得根改成「增量式」
- 現況風險：
  - 支已先 transformed scoring
  - stem root 再從 transformed slice 吃 full root
- 建議：
  - 若 branch 已 transformed
  - `RootApplication` 只應吃：
    - 該 slice 的增量份額
    - 或根據 `PRIMARY / SECONDARY / RESIDUAL` 設定 capped multiplier

### 規則 B：同一 transform group 對同一 stem 只允許一個主根
- 例如 `申酉戌三會化金`
  - 對 `庚/辛`
  - 只選最強的一支做 `LU / MIDDLE`
  - 其餘若仍要保留，只能降成 `MINOR` 或 `TOMB`

### 規則 C：透干增值不可再次全量吃 transform energy
- transformed slice 已經提高 branch contribution 後
- transparency bonus 應：
  - 用該 slice 的 `transparencyBaseFactor`
  - 再乘 group-level cap
- 不可再把整個 branch position-adjusted value 當基底

### 規則 D：共享專氣支雙半合、三會、三合要有 group cap
- 例如：
  - `戌午戌`
  - `申酉戌`
- 應把整個 group 當成一組 energy budget
- 不是每支各自獨立再對 stems 發 full benefit

---

## 目前 code 最可能重複計算的位置

### 1. `build_transformed_branch_qi_slices`
- 檔案：
  - `src-tauri/src/quant_model.rs`
- 風險：
  - transformed slice 已先做
    - `base_element_score * factor`
    - `position_adjusted`
  - 這已是一次放大
- 若後續 stem root / transparency 再吃 full transformed slice，
  就容易雙算

### 2. `resolve_stem_root`
- 檔案：
  - `src-tauri/src/quant_model.rs`
- 風險：
  - 遇到 `branch_transforms` 中已有該支化局結果
  - 會遍歷所有 transformed slices
  - 對每個符合 element 的 slice 都可能建立 `LU` root candidate
- 這對 `W先生` 的 `申酉戌三會化金` 特別危險：
  - `庚辛` 可能從多支重複拿主根

### 3. `score_stems`
- 檔案：
  - `src-tauri/src/quant_model.rs`
- 風險：
  - 每個 branch 都跑一次 `resolve_stem_root`
  - bonus 直接累加
- 若沒有 group-level dedup，
  同一合局可對同一天干累積多次 root bonus

### 4. `resolve_transformed_branch_transparencies`
- 呼叫位置：
  - `src-tauri/src/quant_model.rs`
  - Rust function 名稱：`resolve_transformed_branch_transparencies`
- 風險：
  - transformed branch 的透干已取 `transparency_base`
  - 若同時多 slices 對同一 visible stem 生效，
    也可能重複放大

### 5. 祿地列分改寫
- 檔案：
  - `src-tauri/src/quant_model.rs`
  - 目前 Rust 版沒有獨立 function，祿地改寫邏輯在 `score_branch` / `build_natural_branch_qi_slices` 的列分組裝附近檢查。
- 風險：
  - `祿` row 已把 branch 直接提升到 `+100`
  - 若 stem root 又把這個 branch 當 full lu root，
    需要注意是否已間接重複表達同一股祿氣

### 6. transform resolver helper
- 檔案：
  - `src-tauri/src/quant_model.rs`
  - 重點 function：`create_branch_transform`、`merge_transforms`、`resolve_*_factor`
- 風險：
  - 若 transform slices 的 factor 設計本身已偏滿，
  - score engine 再不去重，就會把章節後補的強化全部放大成「超額收益」

---

## 建議修正順序

### 第一步：先補完整回歸
- 先把第十篇至少兩盤鎖進 calculator regression：
  - `K先生`
  - `W先生`
- 目標不是一開始就完全命中，
  而是建立防線，避免再漂得更遠

### 第二步：實作 transform-group dedup
- 對 `resolveStemRoot` 增加概念：
  - 同一 `transform group id`
  - 同一 visible stem
  - 只能取一次主根

### 第三步：把 transformed transparency 改成 capped incremental basis
- transparency 基底改成：
  - slice 增量
  - 而不是 branch full transformed value

### 第四步：再回頭校準 K/W
- 看 `K` 的 `戌午半合`
- 看 `W` 的 `申酉戌三會`
- 調整到接近書上 `+292 / +282`

---

## 結論
- 書上不是沒有用合局加成公式
- 而是：
  - 用了合局公式
  - 同時用了重分配
  - 也限制了後續得根、透干的取用上限

- 目前程式最可能的偏差，不是「判局錯」，
  而是：
  - **同一股 transformed energy 在地支、得根、透干被重複算了兩到三次**

- 因此修法不是回退所有合局，
  而是補：
  - `transform-group dedup`
  - `incremental root`
  - `capped transparency`
