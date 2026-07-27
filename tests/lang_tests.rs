use docpack::lang::I18n;

#[test]
fn test_lang_load_languages() {
    let i18n = I18n::new();
    assert!(!i18n.available_langs().is_empty());
}

#[test]
fn test_lang_get_string() {
    let i18n = I18n::new();
    let s = i18n.get("app_name");
    assert!(!s.is_empty());
    assert_ne!(s, "app_name");
}

#[test]
fn test_lang_switch_lang() {
    let mut i18n = I18n::new();
    i18n.set_lang("en");
    let en = i18n.get("app_name").to_string();
    i18n.set_lang("zh-CN");
    let zh = i18n.get("app_name").to_string();
    assert_ne!(en, zh);
}

#[test]
fn test_lang_missing_key() {
    let i18n = I18n::new();
    assert_eq!(i18n.get("nonexistent_key"), "nonexistent_key");
}
