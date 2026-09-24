use crate::localization::{Catalog, CatalogSet, MessageValue};
use std::sync::OnceLock;
static CATALOGS: OnceLock<CatalogSet> = OnceLock::new();
fn catalogs() -> &'static CatalogSet {
    CATALOGS.get_or_init(|| {
        CatalogSet::new(
            vec![
                Catalog::new("en-US", include_str!("../locales/en-US.ftl"))
                    .expect("bundled messages"),
                Catalog::new("zh-CN", include_str!("../locales/zh-CN.ftl"))
                    .expect("bundled messages"),
                Catalog::new("zh-TW", include_str!("../locales/zh-TW.ftl"))
                    .expect("bundled messages"),
            ],
            "en-US",
        )
        .expect("bundled locale")
    })
}
pub fn select(locale: &str) {
    let _ = catalogs().select(locale);
}
pub fn tr(key: &str) -> &'static str {
    catalogs()
        .text(&format!("cardo-{key}"))
        .expect("bundled message")
}
pub fn tf(key: &str, args: &[(&str, MessageValue<'_>)]) -> String {
    catalogs()
        .format(&format!("cardo-{key}"), args)
        .expect("bundled message arguments")
}
