//! Référencement : données structurées (JSON-LD), robots.txt et sitemap.xml.

use crate::docs;
use crate::github::Release;
use crate::i18n::{Lang, Tr};
use serde_json::json;

/// Noms sous lesquels on cherche le launcher.
const ALTERNATE_NAMES: [&str; 4] = [
    "Misty MC",
    "MistyMC",
    "Misty Launcher",
    "Misty Minecraft Launcher",
];

/// Sérialise du JSON-LD pour l'insérer dans une balise <script> sans risque de la refermer.
fn to_script(value: serde_json::Value) -> String {
    value.to_string().replace("</", "<\\/")
}

/// JSON-LD de la page d'accueil : le site, l'organisation et le logiciel.
pub fn home_jsonld(site: &str, tr: &Tr, repo: &str, latest: Option<&Release>) -> String {
    let org = json!({
        "@type": "Organization",
        "@id": format!("{site}/#organization"),
        "name": "Misty-MC",
        "alternateName": "Misty MC",
        "url": format!("{site}/"),
        "logo": format!("{site}/assets/icone.png"),
        "sameAs": ["https://github.com/Misty-MC", format!("https://github.com/{repo}")],
    });
    let website = json!({
        "@type": "WebSite",
        "@id": format!("{site}/#website"),
        "name": "Misty MC",
        "alternateName": ["Misty", "Misty Launcher"],
        "url": format!("{site}/"),
        "inLanguage": Lang::ALL.iter().map(|l| l.code()).collect::<Vec<_>>(),
        "publisher": { "@id": format!("{site}/#organization") },
    });
    let mut app = json!({
        "@type": "SoftwareApplication",
        "name": "Misty",
        "alternateName": ALTERNATE_NAMES,
        "description": tr.description,
        "url": format!("{site}/"),
        "image": format!("{site}/assets/screens/en/home.png"),
        "screenshot": format!("{site}/assets/screens/en/home.png"),
        "applicationCategory": "GameApplication",
        "applicationSubCategory": "Minecraft launcher",
        "operatingSystem": "Windows 10, Windows 11",
        "downloadUrl": format!("{site}/download/setup"),
        "license": "https://opensource.org/licenses/MIT",
        "isAccessibleForFree": true,
        "offers": { "@type": "Offer", "price": "0", "priceCurrency": "EUR" },
        "publisher": { "@id": format!("{site}/#organization") },
        "codeRepository": format!("https://github.com/{repo}"),
        "programmingLanguage": "Rust",
    });
    if let Some(r) = latest {
        app["softwareVersion"] = json!(r.tag_name.trim_start_matches('v'));
        if let Some(date) = &r.published_at {
            app["dateModified"] = json!(date);
        }
    }
    to_script(json!({ "@context": "https://schema.org", "@graph": [org, website, app] }))
}

/// JSON-LD d'une page de documentation : article technique + fil d'Ariane.
pub fn doc_jsonld(site: &str, tr: &Tr, page: &docs::DocPage, lang: Lang) -> String {
    let url = format!("{site}/docs/{}?lang={}", page.slug, lang.code());
    to_script(json!({
        "@context": "https://schema.org",
        "@graph": [
            {
                "@type": "TechArticle",
                "headline": page.title,
                "description": page.description,
                "url": url,
                "inLanguage": lang.code(),
                "about": { "@type": "SoftwareApplication", "name": "Misty", "alternateName": ALTERNATE_NAMES },
                "publisher": { "@id": format!("{site}/#organization") },
            },
            {
                "@type": "BreadcrumbList",
                "itemListElement": [
                    { "@type": "ListItem", "position": 1, "name": "Misty MC", "item": format!("{site}/") },
                    { "@type": "ListItem", "position": 2, "name": tr.docs_label, "item": format!("{site}/docs") },
                    { "@type": "ListItem", "position": 3, "name": page.title, "item": url },
                ],
            },
        ],
    }))
}

pub fn robots_txt(site: &str) -> String {
    format!(
        "User-agent: *\nAllow: /\nDisallow: /download/\nDisallow: /api/\nDisallow: /verify\nDisallow: /activity\n\nSitemap: {site}/sitemap.xml\n"
    )
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Sitemap : chaque page, dans chaque langue, avec ses variantes hreflang.
pub fn sitemap_xml(site: &str) -> String {
    let mut paths: Vec<String> = vec![
        "/".into(),
        "/docs".into(),
        "/patch-notes".into(),
        "/shop".into(),
        "/status".into(),
    ];
    paths.extend(
        docs::get(Lang::En)
            .pages
            .iter()
            .map(|p| format!("/docs/{}", p.slug)),
    );

    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\" xmlns:xhtml=\"http://www.w3.org/1999/xhtml\">\n",
    );
    for path in &paths {
        let base = format!("{site}{path}");
        let alternates: String = Lang::ALL
            .iter()
            .map(|l| {
                format!(
                    "    <xhtml:link rel=\"alternate\" hreflang=\"{}\" href=\"{}\"/>\n",
                    l.code(),
                    escape_xml(&format!("{base}?lang={}", l.code()))
                )
            })
            .chain(std::iter::once(format!(
                "    <xhtml:link rel=\"alternate\" hreflang=\"x-default\" href=\"{}\"/>\n",
                escape_xml(&base)
            )))
            .collect();
        for l in Lang::ALL {
            let priority = match path.as_str() {
                "/" => "1.0",
                "/docs" | "/patch-notes" => "0.8",
                _ => "0.6",
            };
            out.push_str(&format!(
                "  <url>\n    <loc>{}</loc>\n{alternates}    <priority>{priority}</priority>\n  </url>\n",
                escape_xml(&format!("{base}?lang={}", l.code()))
            ));
        }
    }
    out.push_str("</urlset>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sitemap_lists_every_page_in_every_language() {
        let xml = sitemap_xml("https://example.com");
        let pages = 5 + docs::get(Lang::En).pages.len();
        assert_eq!(xml.matches("<url>").count(), pages * Lang::ALL.len());
        assert!(xml.contains("<loc>https://example.com/docs/installation?lang=fr</loc>"));
        assert!(xml.contains("hreflang=\"x-default\" href=\"https://example.com/shop\""));
    }

    #[test]
    fn jsonld_is_valid_and_names_misty_mc() {
        let s = home_jsonld("https://example.com", Lang::En.tr(), "Misty-MC/Misty", None);
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert!(v.to_string().contains("Misty MC"));
        assert!(!s.contains("</"));
    }
}
