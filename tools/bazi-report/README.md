# bazi-report：命盤報告工具與 MCP

把 App 的排盤和分析規則包成指令列工具與 MCP server，讓 Claude（或你自己）一行就拿到完整的命盤分析資料，不用開 App、不用自己拼資料。

**只讀 App 的程式，不改 App。** Rust 核心（`src-tauri/src/`）會被複製到 `tools/bazi-report/.build/`，拿掉 `#[tauri::command]` 後單獨編譯，所以不需要 Tauri 或 WebView；前端規則（`src/utils/*.ts`）用 vite 直接載入。App 程式改了以後，下次執行會自動重新編譯（第一次約 30 秒，之後不到 1 秒）。

需要：Node（專案的 `node_modules` 要已安裝）和 Rust（cargo）。

## 命盤報告

```bash
node tools/bazi-report/report.mjs --date 1970-11-06 --time 05:00 --gender F --years 2024-2030
node tools/bazi-report/report.mjs --date 59-10-08 --era roc --calendar lunar --time 05:00 --gender 女
node tools/bazi-report/report.mjs --pillars "庚戌 丙戌 庚寅 己卯" --gender F    # 會回推公曆出生時間
npm run report -- --date 1970-11-06 --time 05:00 --gender F --detail
```

| 選項 | 說明 |
|---|---|
| `--date` `--time` | 出生日期時間；農曆加 `--calendar lunar`（閏月加 `--leap`），民國年加 `--era roc` |
| `--pillars` | 直接給四柱（年 月 日 時）；有兩個候選年份時用 `--birthYear` 指定 |
| `--gender` | `F`／`女` 或 `M`／`男` |
| `--years` | 要看的流年，`2026` 或 `2024-2030`；預設今年 |
| `--format json` | 輸出完整 JSON（預設是精簡 Markdown） |
| `--detail` | 列出每個提示的說明和量化計分明細 |
| `--yunSect` | 起運流派，預設 1（和 App 表單一樣） |

報告內容：
- App 排法命盤表（時 日 月 年、十神、五行小字、藏干本氣置中、藏干十神）和五行個數
- 課堂三步法：取格、強弱三項、喜忌方案、合併喜忌，以及天干、地支、同柱、複核四張卡片的結論
- 量化五行力、陸致極強弱、基礎教程格局用神、十神洩天機組合、蘇民峰特別格局
- 婚姻・財運・健康面板（寒暖燥濕、調候、弱臟……）
- 每步大運、每個流年：十神、課堂喜忌、量化分數、三合三會填實、App 的提示標籤

兩個 App 沒有的補充：
1. **量化分數的讀法**：App「傾向」欄只看分數正負（正分＝偏吉），但正分代表幫日主變強，對身強的盤其實不利。報告改看「作用後命局」離 0 的遠近：靠近 0 寫「趨中和（有利）」，離更遠寫「更偏（不利）」。從格要反過來看。
2. **填實**：大運、流年湊成原局沒有的三合、三會時會標出來（例如原局寅戌，流年午 → 寅午戌三合火局）。

## 查筆記、找相似命例

```bash
node tools/bazi-report/notes.mjs search 土多金埋 庚          # 關鍵字都要出現在同一段
node tools/bazi-report/notes.mjs similar "庚戌 丙戌 庚寅 己卯"  # 依日干、月令、日柱、同字數排序
```

預設搜尋 `docs/books/`、`上課筆記整理.md`、`bazi-skills/`。要加其他資料夾，設環境變數 `BAZI_NOTES_DIRS`（Windows 用 `;` 分隔），例如 `E:\Data\src\bazi-research`。相似命例會從筆記裡用 App 排法畫的命盤表（約 1,900 張）中挑。

## MCP

專案根目錄的 `.mcp.json` 已經註冊了 `bazi` server，在這個資料夾開 Claude Code 就會看到。其他 MCP 用戶端（例如 IDEA 的 AI Assistant）手動加：

```json
{ "command": "node", "args": ["E:\\Data\\src\\JavaPrj\\bazi\\tools\\bazi-report\\mcp.mjs"] }
```

提供三個工具：

| 工具 | 用途 |
|---|---|
| `bazi_report` | 命盤報告（參數同上：date、time、gender、calendar、era、leap、pillars、birthYear、years、format、detail） |
| `bazi_search_notes` | 關鍵字查書本與上課筆記 |
| `bazi_similar_cases` | 找書中相似命例 |

不需要額外套件，server 直接實作 MCP 的 stdio JSON-RPC。
