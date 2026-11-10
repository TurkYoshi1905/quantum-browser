//! QuantumBrowser — yerleşik reklam & izleyici engelleyici
//! Bilinen reklam/izleyici alan adları gömülü liste + host son ek eşleme.
//! Ağ seviyesinde engelleme Windows'ta WebView2 `WebResourceRequested` ile,
//! kozmetik temizlik ise her platformda initialization-script ile yapılır.

/// Bilinen reklam ve izleyici ağları (parent-domain eşleme: `a.b.com` → `b.com` üzerinden de yakalanır).
const BLOCKED_DOMAINS: &[&str] = &[
    // Google reklam ağı & ölçüm
    "doubleclick.net",
    "googlesyndication.com",
    "googleadservices.com",
    "googletagmanager.com",
    "googletagservices.com",
    "google-analytics.com",
    "analytics.google.com",
    "stats.g.doubleclick.net",
    "adservice.google.com",
    "pagead2.googlesyndication.com",
    "2mdn.net",
    "googleadsserving.cn",
    // Büyük exchange'ler / SSP'ler
    "adnxs.com",
    "adsrvr.org",
    "rubiconproject.com",
    "pubmatic.com",
    "openx.net",
    "casalemedia.com",
    "3lift.com",
    "bidswitch.net",
    "smartadserver.com",
    "adform.net",
    "adform.com",
    "criteo.com",
    "criteo.net",
    "advertising.com",
    "yieldmanager.com",
    "serving-sys.com",
    "adtechus.com",
    "aexp.com",
    "amazon-adsystem.com",
    "aan.amazon.com",
    "media-amazon.com",
    // Ölçüm & retargeting
    "scorecardresearch.com",
    "quantserve.com",
    "demdex.net",
    "omtrdc.net",
    "everesttech.net",
    "adsrvr.net",
    "adsafeprotected.com",
    "moatads.com",
    "zedo.com",
    "advertising.com",
    "adroll.com",
    "adsymptotic.com",
    "perfectaudience.com",
    "bidr.io",
    "turn.com",
    "bluekai.com",
    "krxd.net",
    "tapad.com",
    "onthe.io",
    "emxdgt.com",
    // Sosyal pikseller
    "pixel.facebook.com",
    "connect.facebook.net",
    "analytics.twitter.com",
    "static.ads-twitter.com",
    "ads-twitter.com",
    "bat.bing.com",
    "snap.licdn.com",
    "dc.ads.linkedin.com",
    "analytics.tiktok.com",
    "ads.tiktok.com",
    "alb.reddit.com",
    "pixel.redditmedia.com",
    // İçerik tavsiye ağları
    "taboola.com",
    "outbrain.com",
    "mgid.com",
    "revcontent.com",
    "contentad.net",
    // Diğer izleyiciler
    "hotjar.com",
    "mouseflow.com",
    "fullstory.com",
    "logrocket.com",
    "clarity.ms",
    "c.clarity.ms",
    "newrelic.com",
    "nr-data.net",
    "segment.io",
    "segment.com",
    "mixpanel.com",
    "amplitude.com",
    "heap.io",
    "heapanalytics.com",
    "branch.io",
    "branch.app",
    "adjust.com",
    "appsflyer.com",
    "kochava.com",
    "mxpnl.com",
    // Mobil/ reklam altyapısı
    "adcolony.com",
    "unityads.unity3d.com",
    "ads.unityads.unity3d.com",
    "applovin.com",
    "mopub.com",
    "vungle.com",
    "startapp.com",
    "inmobi.com",
    "smaato.net",
    "supersonicads.com",
    "fyber.com",
    "adtiming.com",
    // RTB & jenerik
    "adglare.net",
    "adglare.com",
    "admedo.com",
    "adshooter.co",
    "adtrafficquality.google",
    "pagead.l.doubleclick.net",
    "cm.g.doubleclick.net",
];

/// Kozmetik filtreleme: yaygın reklam konteynerlarını gizler.
/// Her sayfada initialization-script olarak çalıştırılır.
pub const COSMETIC_JS: &str = r#"
(() => {
  if (window.__qbBlockerLoaded) return;
  window.__qbBlockerLoaded = true;
  const css = [
    'ins.adsbygoogle','div[id^="div-gpt-ad"]','div[id^="google_ads"]','div[id^="ad-"]',
    'div[class^="ad-"]','div[class*=" advert"]','div.ad-banner','div.ads-container',
    '#ads','#ad-container','#ad_banner','.adsbygoogle','.ad-slot','.ad-wrapper',
    '.sponsor-banner','.sponsored-content','.taboola-widget','._cdc_p_hidden',
    'iframe[src*="doubleclick.net"]','iframe[src*="googlesyndication.com"]',
    'iframe[src*="adnxs.com"]','iframe[src*="taboola.com"]','iframe[src*="outbrain.com"]',
    '[data-ad-slot]','[data-ad]','.mgid-iframe','div[class*="OUTBRAIN"]','.trc_rbox'
  ].join(',') + ' { display: none !important; visibility: hidden !important; height: 0 !important; max-height: 0 !important; overflow: hidden !important; }';
  const install = () => {
    const root = document.head || document.documentElement;
    if (!root || document.getElementById('qb-blocker-style')) return;
    const style = document.createElement('style');
    style.id = 'qb-blocker-style';
    style.textContent = css;
    root.appendChild(style);
  };
  install();
  const mo = new MutationObserver(install);
  mo.observe(document.documentElement, { childList: true, subtree: true });
})();
"#;

/// Bir URL'nin engellenip engellenmeyeceğine karar ver.
/// Yalnızca host eşleşmesi yapar; yol/parametreye bakmaz (hızlı + öngörülebilir).
pub fn is_blocked(url: &str) -> bool {
    let host = match host_of(url) {
        Some(h) => h,
        None => return false,
    };
    BLOCKED_DOMAINS
        .iter()
        .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
}

fn host_of(url: &str) -> Option<String> {
    let after_scheme = url.split_once("://")?.1;
    let without_user = match after_scheme.rsplit_once('@') {
        Some((_, rest)) => rest,
        None => after_scheme,
    };
    let authority = without_user.split(['/', '?', '#']).next()?;
    let host = authority.split(':').next()?.trim();
    if host.is_empty() {
        return None;
    }
    Some(host.to_ascii_lowercase())
}

/// Basit host çıkarımı (testler ve sekme başlığı için herkese açık).
pub fn domain_of(url: &str) -> String {
    host_of(url).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_ad_networks() {
        assert!(is_blocked(
            "https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js"
        ));
        assert!(is_blocked("https://ads.doubleclick.net/gampad/x.js"));
        assert!(is_blocked("https://a.pubmatic.com/b"));
        assert!(is_blocked("https://pixel.facebook.com/tr?id=1"));
        assert!(is_blocked("https://taboola.com/sync"));
    }

    #[test]
    fn allows_normal_sites() {
        assert!(!is_blocked("https://www.youtube.com/watch?v=abc"));
        assert!(!is_blocked("https://github.com/rust-lang/rust"));
        assert!(!is_blocked("https://www.google.com/search?q=rust"));
        assert!(!is_blocked("https://wikipedia.org/wiki/Rust"));
    }

    #[test]
    fn handles_tricky_urls() {
        assert!(is_blocked("https://user:pass@ad.doubleclick.net/x"));
        assert!(is_blocked("https://AD.doubleclick.net/x"));
        assert!(!is_blocked("not a url"));
        assert!(!is_blocked("https://"));
    }
}
