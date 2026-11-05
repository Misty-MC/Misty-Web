//! Mise en forme des dates et des tailles selon la langue.

use crate::i18n::Tr;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// « 20 octobre 2026 » ou « October 20, 2026 » selon `date_order`.
pub fn date(tr: &Tr, iso: Option<&str>) -> String {
    let Some(d) = iso.and_then(|s| OffsetDateTime::parse(s, &Rfc3339).ok()) else {
        return String::new();
    };
    let month = &tr.months[usize::from(u8::from(d.month())) - 1];
    if tr.date_order == "mdy" {
        format!("{month} {}, {}", d.day(), d.year())
    } else {
        format!("{} {month} {}", d.day(), d.year())
    }
}

/// « 9,0 Mo » / « 9.0 MB ».
pub fn size(tr: &Tr, bytes: u64) -> String {
    let mb = format!("{:.1}", bytes as f64 / 1_048_576.0);
    format!("{} {}", mb.replace('.', &tr.decimal_sep), tr.mb)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Lang;

    #[test]
    fn dates() {
        let iso = Some("2026-10-20T10:00:00Z");
        assert_eq!(date(Lang::Fr.tr(), iso), "20 octobre 2026");
        assert_eq!(date(Lang::En.tr(), iso), "October 20, 2026");
        assert_eq!(date(Lang::Fr.tr(), Some("pas une date")), "");
    }

    #[test]
    fn sizes() {
        assert_eq!(size(Lang::Fr.tr(), 9_437_184), "9,0 Mo");
        assert_eq!(size(Lang::En.tr(), 9_437_184), "9.0 MB");
    }
}
