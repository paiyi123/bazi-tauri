# 量化模型案例索引

這個資料夾用來保存《量化五行力》相關的回歸案例、差距案例與理論筆記。

這些文件是從原本服務端模型移到 BaZi Desktop 後整理過的工程筆記；現行 App 的量化核心在 `src-tauri/src/quant_model.rs`，前端只顯示 Rust 回傳的量化結果。若筆記提到 benchmark / regression，後續應優先轉成 `cargo test` 可跑的 Rust 測試。

若要先看整體架構與篇章對應，請先讀 `../quant-model-system-guide.md`。

## 目前文件
- `001-1970-03-08-ding-hai.md`
  - 基準負分案例，適合驗證基礎十神、位置權重與藏干比例是否穩定
- `002-jane-zhang-gap-case.md`
  - 書本風格差距案例，適合追蹤祿根、透干與進階修正尚未補齊的部分
- `003-book-case-005-half-combo-fire.md`
  - 第 67 頁案例五，重點是 `寅午半合化火`
- `004-book-case-006-bing-xin-combination.md`
  - 案例六，重點是 `丙辛合而不化`
- `005-book-fourth-part-chong-he-notes.md`
  - 第四篇 `99-122` 頁理論筆記，重點是 `半合 / 六合 / 逢沖 / 多重合化 / 二次合化 / 左右合化局並見`
- `006-book-case-009-half-combo-water.md`
  - 第 79 頁案例，重點是 `子辰半合化水`
- `007-book-case-010-half-combo-not-transform.md`
  - 第 80 頁案例，重點是 `申子合而不化`
- `008-theory-half-vs-six-priority.md`
  - 理論型筆記，說明為何共支時 `半合` 應優先於 `六合`
- `009-book-part5-quantified-combination-force.md`
  - 第五篇 `123-140` 頁量化筆記，整理 `六合 / 半合 / 三合 / 三會` 的合化與不化力量
- `010-book-case-bai-yun-qin-six-combo-gold.md`
  - 白韻琴案例，重點是 `辰酉六合化金` 要拆成 `1.0 金 + 0.8 金 + 0.2 餘氣`
- `011-book-case-special-male-three-meeting-non-transform.md`
  - 特殊命例，重點是 `巳午未三會不化` 的提升量模型，以及 `甲庚隔一位相沖 ×0.875`
- `012-book-case-fake-follow-female-double-six-combination.md`
  - 假從女命案例，重點是 `未午未` 夾合壓過 `寅午` 半合，月令 `午` 只剩 `0.04`
- `013-refactor-architecture-phases.md`
  - 量化模型重構紀錄，整理 phase 過程、目前架構分層，以及後續擴規則建議落點
- `014-book-part8-luck-notes.md`
  - 第八篇 `196-207` 頁筆記，整理大運基本值、`R 值相對影響` 與大運入局判則
- `015-book-part8-luck-examples.md`
  - 第八篇 `208-218` 頁案例節錄，整理張韶穎、許瑋倫、意外凶死男命、U 先生與假從女命的大運例子
- `016-book-part3-yongshen-notes.md`
  - 第三篇 `84-97` 頁筆記，整理「先判強弱、再看見印、最後局部修正」的用神方法與章內範例
- `017-book-part4-advanced-pattern-examples.md`
  - 第四篇 `119-122` 頁 pattern 範例，整理 `二次合化` 與 `左右合化局並見` 的可測試規則
- `018-book-part5-advanced-multi-interaction.md`
  - 第五篇 `141-151` 頁後段筆記，整理 `合合並見 / 專氣支二次合化 / 左右夾沖 / 多局互制`，並標示目前已落地規則與對應回歸測試
- `019-book-part6-cong-follow-notes.md`
  - 第六篇 `152-164` 頁筆記，整理 `從強 / 從弱 / 不從` 的 `±210` 界線、十神主導家族判法，以及 P先生 / 白韻琴 / W先生三個校準案例
- `020-book-part7-jingsuan-examples.md`
  - 第七篇 `165-195` 頁案例筆記，整理 `共享專氣支雙半合`、`雙三合不化`、`三會化木`、`子午沖封祿根` 與 `六合緩沖沖力` 的實作狀態
- `021-book-part9-sha-yin-notes.md`
  - 第九篇 `219-234` 頁筆記，整理 `天干殺印相生`、`地支純氣殺印相生`、`負能轉正` 的成立條件與 M先生 / C先生 / 小沈陽案例
- `022-book-part10-cong-advanced-notes.md`
  - 第十篇 `235-253` 頁筆記，整理 `真從 / 假從` 的 `±260` 界線、真從強可兼用食傷、假從較怕逆運，以及 K先生 / W先生 / 李兆基與兩個反例
- `023-book-part11-gan-zhi-meaning-notes.md`
  - 第十一篇 `254-260` 頁筆記，整理干支、藏干與陽干略強於陰干的理論來源；本篇主要作為理論依據，不直接新增算分規則
- `024-book-part12-famous-birth-hour-notes.md`
  - 第十二篇 `261-275` 頁筆記，整理名人時辰比對方法，並收錄趙薇、歐巴馬、郭晶晶之子等候選時辰案例
- `025-book-part13-god-does-not-play-dice-notes.md`
  - 第十三篇 `276-285` 頁筆記，整理比特命例、真太陽時修正，以及如何對待仍無法完全解釋的特例
- `026-implementation-guidelines-avoid-overcounting.md`
  - 實作準則筆記，整理「避免合局成立後地支、得根、透干重複高估」的工程護欄，並標出目前 code 最可能重複計算的熱點
- `027-chapter-benchmark-regression-cases.md`
  - 章節基準回歸整理，整理哪些案例應保留為 benchmark / regression guardrail，並說明第八篇應保留大運與流年共同裁決的案例

## 後續值得補的案例類型
- 明確的祿根翻盤案例
- 透干放大型案例
- 強沖合次序案例
- 假從與不從邊界案例
- 第四篇中直接比較 `半合 vs 六合` 的書本實盤案例
