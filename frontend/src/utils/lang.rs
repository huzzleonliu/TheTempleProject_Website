/// 支持的语言
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Zh,
    En,
}

impl Lang {
    pub fn toggle(self) -> Self {
        match self {
            Lang::Zh => Lang::En,
            Lang::En => Lang::Zh,
        }
    }
}

const STORAGE_KEY: &str = "lang";

/// 初始化语言：优先 localStorage，其次 navigator.language
pub fn init_lang() -> Lang {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(value)) = storage.get_item(STORAGE_KEY) {
                if let Some(lang) = Lang::from_str(&value) {
                    return lang;
                }
            }
        }

        if let Some(navigator_lang) = window.navigator().language() {
            if let Some(lang) = Lang::from_str(&navigator_lang) {
                return lang;
            }
        }
    }

    Lang::Zh
}

impl Lang {
    pub fn as_str(self) -> &'static str {
        match self {
            Lang::Zh => "zh",
            Lang::En => "en",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.to_ascii_lowercase();
        if s.starts_with("zh") {
            Some(Lang::Zh)
        } else if s.starts_with("en") {
            Some(Lang::En)
        } else {
            None
        }
    }
}

/// 将当前语言写入 localStorage
pub fn persist_lang(lang: Lang) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item(STORAGE_KEY, lang.as_str());
        }
    }
}


