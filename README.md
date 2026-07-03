# 醫事人員執業異動文字產生器 (Medical Personnel Registration Change Generator)

一個簡潔、美觀的單頁 Web 工具，協助醫事人員快速產生申辦執業異動時所需的格式化文字。

## 本地開發與編譯 CSS

本專案使用 Tailwind CSS 進行樣式設計。為了避免在瀏覽器中直接載入 CDN 導致效能降低或產生警告，我們已將 CSS 編譯為靜態檔案。

### 1. 安裝相依套件

如果您需要修改樣式或 HTML 結構，請先確保本地有安裝 Node.js，並在終端機中執行：

```bash
npm install
```

### 2. 開發階段（監聽並自動編譯）

在修改 HTML 檔案中的 Tailwind Class 時，可以執行下方指令，Tailwind CLI 會自動偵測變更並即時更新編譯 `./dist/output.css`：

```bash
npm run watch
```

### 3. 生產發布編譯（壓縮優化）

在準備將專案發布或部署時，請執行以下指令來產出最小化（Minified）的 CSS 檔案：

```bash
npm run build
```

---

## 專案結構

- `index.html` - 主要網頁結構與 JavaScript 互動邏輯。
- `favicon.ico` - 網站圖示（16x16, 32x32, 48x48, 64x64 多尺寸 ICO 格式）。
- `src/input.css` - Tailwind 來源入口 CSS 檔案。
- `dist/output.css` - 編譯完成後的輕量化 CSS 輸出檔案。
- `tailwind.config.js` - Tailwind CSS 設定檔，已配置掃描 `index.html`。
