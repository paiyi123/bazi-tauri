# 筆記上傳 Confluence

把書籍筆記、上課筆記、命例分析上傳到 Confluence「八字命理」空間（Bazi，根頁面 159318050）。

## 分類
- 書籍筆記 → 依作者分組（河洛子、陸致極、吳俊民、潘樂德、蘇民峰）→ 每本書 → 各筆記檔
- 上課筆記（許羽賢老師）→ 課程總結、每週上課筆記、中級二課本、如何取用神
- 命例分析 → chart-reviews 的命盤解讀

另外有：首頁導覽（搜尋框、依主題／分類／作者的入口、最近更新、頁面樹；原本首頁內容會收進最下方的展開區）、「主題索引」（14 個主題，依內容自動判定，主題與關鍵詞在 topics.py）、每頁標籤、每頁上方「本頁目錄」、頁尾上一篇／下一篇。

超過 250 KB 的檔案（命理新論逐頁筆記）依章節（`#` 標題）拆成子頁。Markdown 之間的連結會改成 Confluence 頁面連結，表格與 `<sub>` 五行小字保留。

## 步驟
1. 產生頁面（在有 Python 的環境，需要 `pip install markdown lxml`）：
   `python3 build.py <repo>/docs/books <project-files>/notes out`
   會產生 `out/manifest.json` 和 `out/pages/*.xml`（Confluence storage 格式）。
2. 在 Confluence 建立個人存取權杖，存到 `%USERPROFILE%\.confluence-token`（只有權杖一行）。
3. 試跑：`node tools/confluence-upload/upload.mjs --dry-run`
4. 上傳：`node tools/confluence-upload/upload.mjs`
   同標題的頁面會更新（新版本），沒有的才建立，可以重複執行。
   只更新部分頁面：`--only 命理新論`（標題包含該字串的頁面）。

環境變數：`CONFLUENCE_URL`、`CONFLUENCE_SPACE`、`CONFLUENCE_ROOT`、`CONFLUENCE_TOKEN_FILE`、`BUNDLE_DIR`。
