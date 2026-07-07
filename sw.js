// ═══════════════════════════════════════════════════
// Service Worker — 醫事人員執業異動文字產生器
// 策略：Cache First（核心資源預快取）
//       fonts.css 使用 Stale While Revalidate（檔案較大，背景更新）
// ═══════════════════════════════════════════════════

// 快取版本號（每次更新內容時遞增此值）
const CACHE_VERSION = 'v1';
const CACHE_NAME = `medgen-cache-${CACHE_VERSION}`;

// 預快取的核心資源（安裝時一次性快取）
const PRECACHE_URLS = [
    './',
    './index.html',
    './favicon.ico',
    './manifest.json',
    './icons/icon-192.png',
    './icons/icon-512.png'
];

// 大型資源（使用 Stale While Revalidate，不預快取）
const LARGE_RESOURCES = [
    'fonts.css'
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

// ─── Fetch Event ───
self.addEventListener('fetch', (event) => {
    // 只處理 GET 請求
    if (event.request.method !== 'GET') return;

    const url = new URL(event.request.url);

    // 判斷是否為大型資源 → Stale While Revalidate
    const isLargeResource = LARGE_RESOURCES.some(r => url.pathname.endsWith(r));

    if (isLargeResource) {
        // Stale While Revalidate：立即回傳快取，背景更新
        event.respondWith(
            caches.open(CACHE_NAME).then(cache =>
                cache.match(event.request).then(cached => {
                    const fetchPromise = fetch(event.request)
                        .then(response => {
                            if (response && response.status === 200) {
                                cache.put(event.request, response.clone());
                            }
                            return response;
                        })
                        .catch(() => cached);

                    return cached || fetchPromise;
                })
            )
        );
        return;
    }

    // Cache First：優先快取，未命中才請求網路
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
                        caches.open(CACHE_NAME)
                            .then(cache => cache.put(event.request, clone));
                        return response;
                    });
            })
            .catch(() => {
                // 離線且無快取時的 fallback
                if (event.request.destination === 'document') {
                    return caches.match('./index.html');
                }
            })
    );
});
