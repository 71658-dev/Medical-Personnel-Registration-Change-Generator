// ═══════════════════════════════════════════════════
// Service Worker 註冊 — 醫事人員執業異動文字產生器
//
// 刻意獨立於 WASM 之外：註冊不必等 module 下載完成，
// 且 Cache API 的暖身邏輯屬於 JS 領域，放進 main.rs 只會增加 interop。
// 'sw.js' 相對於文件的 base URL 解析，因此 GitHub Pages 的
// --public-url 子路徑會自動對上，scope 也會落在同一層。
// ═══════════════════════════════════════════════════
(function () {
    if (!('serviceWorker' in navigator)) return;

    // 把本頁實際載入的同源資源交給 worker 補快取。
    // 首次造訪時這些請求發生在 worker 取得控制權之前，不會被 fetch 事件攔截。
    function warmCache() {
        var worker = navigator.serviceWorker.controller;
        if (!worker) return;
        if (!window.performance || !performance.getEntriesByType) return;

        var urls = performance.getEntriesByType('resource')
            .map(function (entry) { return entry.name; })
            .filter(function (name) { return name.indexOf(location.origin) === 0; });

        if (urls.length) worker.postMessage({ type: 'WARM_CACHE', urls: urls });
    }

    window.addEventListener('load', function () {
        navigator.serviceWorker.register('sw.js').then(function () {
            // 首次造訪：clients.claim() 之後才會有 controller
            navigator.serviceWorker.addEventListener('controllerchange', warmCache);
            // 後續造訪：controller 已存在，ready 後直接暖身
            navigator.serviceWorker.ready.then(warmCache);
        }).catch(function (err) {
            console.warn('[medgen] service worker 註冊失敗：', err);
        });
    });
})();
