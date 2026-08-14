// ═══════════════════════════════════════════════════
// Service Worker — 醫事人員執業異動文字產生器
// 策略：HTML 導覽用 Network First（新版部署立即生效）
//       其餘資源用 Cache First（Trunk 已對檔名加雜湊，快取不會過期）
// 註冊來源：register-sw.js
// ═══════════════════════════════════════════════════

// 快取版本號（每次更新內容時遞增此值）
const CACHE_VERSION = 'v3';
const CACHE_NAME = `medgen-cache-${CACHE_VERSION}`;

// 預快取的核心資源（安裝時一次性快取）
// 註：Trunk 產生的雜湊檔名（index-<hash>.js、index_bg-<hash>.wasm、
//     style-<hash>.css）在此無從得知，改由 register-sw.js 於首次載入後
//     透過 WARM_CACHE 訊息補上。
const PRECACHE_URLS = [
    './',
    './index.html',
    './favicon.ico',
    './manifest.json',
    './icons/icon-192.png',
    './icons/icon-512.png'
];

// ─── Install Event ───
// 安裝時預快取所有核心資源
self.addEventListener('install', (event) => {
    event.waitUntil(
        caches.open(CACHE_NAME)
            .then(cache => cache.addAll(PRECACHE_URLS))
            .then(() => self.skipWaiting()) // 立即啟用新版本
    );
});

// ─── Activate Event ───
// 啟用時清除舊版本快取
self.addEventListener('activate', (event) => {
    event.waitUntil(
        caches.keys()
            .then(keys => Promise.all(
                keys.filter(key => key !== CACHE_NAME)
                    .map(key => caches.delete(key))
            ))
            .then(() => self.clients.claim()) // 立即控制所有頁面
    );
});

// ─── Message Event ───
// 首次載入時，頁面的請求早於 service worker 取得控制權而未被攔截，
// 因此雜湊資源不在快取中。由頁面回報實際載入的 URL 後在此補快取，
// 使用者不必重新整理就能離線使用。
self.addEventListener('message', (event) => {
    const data = event.data;
    if (!data || data.type !== 'WARM_CACHE' || !Array.isArray(data.urls)) return;

    event.waitUntil(
        caches.open(CACHE_NAME).then(cache =>
            Promise.all(data.urls.map(url =>
                cache.match(url).then(hit =>
                    // 已在快取中就跳過；個別失敗不影響其他資源
                    hit ? null : cache.add(url).catch(() => null)
                )
            ))
        )
    );
});

// ─── Fetch Event ───
self.addEventListener('fetch', (event) => {
    // 只處理 GET 請求
    if (event.request.method !== 'GET') return;

    // 跨來源請求一律直接走網路，不進快取
    const url = new URL(event.request.url);
    if (url.origin !== self.location.origin) return;

    const isNavigation =
        event.request.mode === 'navigate' || event.request.destination === 'document';

    if (isNavigation) {
        // Network First：index.html 未加雜湊，若走 Cache First，
        // 新版部署後使用者會一直停留在舊版。
        event.respondWith(
            fetch(event.request)
                .then(response => {
                    if (response && response.status === 200 && response.type === 'basic') {
                        const clone = response.clone();
                        caches.open(CACHE_NAME).then(cache => cache.put(event.request, clone));
                    }
                    return response;
                })
                .catch(() =>
                    // 離線：先找該網址的快取，再退回 app shell
                    caches.match(event.request).then(cached => cached || caches.match('./index.html'))
                )
        );
        return;
    }

    // Cache First：其餘資源皆由 Trunk 加上內容雜湊，命中即為正確版本
    event.respondWith(
        caches.match(event.request)
            .then(cached => {
                if (cached) return cached;
                return fetch(event.request)
                    .then(response => {
                        // 不快取非成功的回應或非同源請求
                        if (!response || response.status !== 200 || response.type !== 'basic') {
                            return response;
                        }
                        // 複製回應並存入快取
                        const clone = response.clone();
                        caches.open(CACHE_NAME).then(cache => cache.put(event.request, clone));
                        return response;
                    });
            })
    );
});
