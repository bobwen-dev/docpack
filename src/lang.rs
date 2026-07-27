use std::collections::HashMap;

pub struct I18n {
    current_lang: String,
    translations: HashMap<&'static str, &'static str>,
}

macro_rules! lang_map {
    ($($key:ident = $val:expr),* $(,)?) => {
        [$( (stringify!($key), $val) ),*]
    };
}

const EN: &[(&str, &str)] = &lang_map! {
    app_name = "DocPack",
    app_desc = "Pack an entire source code directory into a single docx file",
    pack = "Pack",
    unpack = "Unpack",
    select_path = "Select path or drag files here",
    drop_hint = "Drop files/folders here",
    output_path = "Output path",
    browse = "Browse",
    exclude_file = "Exclude file (.docpackignore)",
    packing = "Packing...",
    unpacking = "Unpacking...",
    done = "Done!",
    error = "Error",
    success = "Success",
    settings = "Settings",
    language = "Language",
    about = "About",
    version = "Version",
    install_context_menu = "Install context menu",
    uninstall_context_menu = "Uninstall context menu",
    no_text_files = "No text files found",
    file_count = "Files to pack: {count}",
    binary_skipped = "Binary files skipped: {count}",
    output_saved = "Output saved to {path}",
    extracted_to = "Extracted to {path}",
    cli_pack_usage = "Pack files into DOCX",
    cli_unpack_usage = "Unpack DOCX to files",
    cli_exclude = "Exclude file path",
    cli_output = "Output path",
    cli_lang = "Language code",
    first_run_title = "Welcome to DocPack",
    first_run_message = "Would you like to install the context menu for quick access?",
    first_run_install = "Install context menu",
    first_run_skip = "Skip for now",
    encoding = "Local Encoding",
    encoding_desc = "Set encodings for reading text files with non-UTF-8 encoding",
    encoding_hint = "One encoding per line",
    encoding_hint_placeholder = "Enter encodings, one per line",
    tab_exclude = "Exclude",
    tab_language = "Language",
    tab_encoding = "Encoding",
    tab_context_menu = "Context Menu",
    context_menu = "Context Menu",
    context_menu_desc = "Add 'Pack with DocPack' to folders and files, 'Unpack DOCX here' to .docx files",
    tab_about = "About",
    exclude_desc = "Files matching these glob rules will be excluded from packing",
    exclude_hint = "One rule per line",
    exclude_hint_placeholder = "Enter patterns, one per line",
    language_desc = "Change the display language of the application",
    language_hint = "Select display language",
    close = "Close",
    save = "Save",
    cancel = "Cancel",
    scanning = "Scanning...",
    pack_done = "Done: {count} files -> {path}",
    ok = "OK",
    no_source_paths = "No source paths selected",
    extracted_details = "Extracted {count} files to {path}",
    context_menu_installed = "Context menu installed",
    context_menu_uninstalled = "Context menu uninstalled",
};

const ZH_CN: &[(&str, &str)] = &lang_map! {
    app_name = "文档打包工具",
    app_desc = "可将整个软件源代码目录打包为单个 docx 文件",
    pack = "打包",
    unpack = "解包",
    select_path = "选择路径或将文件拖拽至此",
    drop_hint = "将文件/文件夹拖拽到此处",
    output_path = "输出路径",
    browse = "浏览",
    exclude_file = "排除文件 (.docpackignore)",
    packing = "正在打包...",
    unpacking = "正在解包...",
    done = "完成！",
    error = "错误",
    success = "成功",
    settings = "设置",
    language = "语言",
    about = "关于",
    version = "版本",
    install_context_menu = "安装右键菜单",
    uninstall_context_menu = "卸载右键菜单",
    no_text_files = "未找到文本文件",
    file_count = "待打包文件数：{count}",
    binary_skipped = "跳过二进制文件：{count}",
    output_saved = "输出已保存至 {path}",
    extracted_to = "已解压至 {path}",
    cli_pack_usage = "将文件打包为 DOCX",
    cli_unpack_usage = "从 DOCX 解包文件",
    cli_exclude = "排除文件路径",
    cli_output = "输出路径",
    cli_lang = "语言代码",
    first_run_title = "欢迎使用 DocPack",
    first_run_message = "是否安装右键菜单以便快速访问？",
    first_run_install = "安装右键菜单",
    first_run_skip = "暂不安装",
    encoding = "本地编码",
    encoding_desc = "设置读取非 UTF-8 编码文本文件时使用的编码",
    encoding_hint = "每行一个编码",
    encoding_hint_placeholder = "输入编码，每行一个",
    tab_exclude = "排除列表",
    tab_language = "语言",
    tab_encoding = "编码",
    tab_context_menu = "关联菜单",
    context_menu = "关联菜单",
    context_menu_desc = "在文件夹和文件上添加'打包'，在 docx 上添加'解包'",
    tab_about = "关于",
    exclude_desc = "匹配这些 glob 规则的文件将被排除在打包之外",
    exclude_hint = "每行一个规则",
    exclude_hint_placeholder = "输入排除模式，每行一个",
    language_desc = "更改应用程序的显示语言",
    language_hint = "选择显示语言",
    close = "关闭",
    save = "保存",
    cancel = "取消",
    scanning = "正在扫描...",
    pack_done = "完成：{count} 个文件 -> {path}",
    ok = "确定",
    no_source_paths = "未选择源路径",
    extracted_details = "已解压 {count} 个文件到 {path}",
    context_menu_installed = "右键菜单已安装",
    context_menu_uninstalled = "右键菜单已卸载",
};

const ZH_TW: &[(&str, &str)] = &lang_map! {
    app_name = "檔案打包工具",
    app_desc = "可將整個軟體源代碼目錄打包為單個 docx 文件",
    pack = "打包",
    unpack = "解包",
    select_path = "選擇路徑或拖曳檔案至此",
    drop_hint = "將檔案/資料夾拖曳至此",
    output_path = "輸出路徑",
    browse = "瀏覽",
    exclude_file = "排除檔案 (.docpackignore)",
    packing = "正在打包...",
    unpacking = "正在解包...",
    done = "完成！",
    error = "錯誤",
    success = "成功",
    settings = "設定",
    language = "語言",
    about = "關於",
    version = "版本",
    install_context_menu = "安裝右鍵選單",
    uninstall_context_menu = "解除安裝右鍵選單",
    no_text_files = "未找到文字檔案",
    file_count = "待打包檔案數：{count}",
    binary_skipped = "跳過二進制檔案：{count}",
    output_saved = "輸出已儲存至 {path}",
    extracted_to = "已解壓至 {path}",
    cli_pack_usage = "將檔案打包為 DOCX",
    cli_unpack_usage = "從 DOCX 解包檔案",
    cli_exclude = "排除檔案路徑",
    cli_output = "輸出路徑",
    cli_lang = "語言代碼",
    first_run_title = "歡迎使用 DocPack",
    first_run_message = "是否安裝右鍵選單以便快速存取？",
    first_run_install = "安裝右鍵選單",
    first_run_skip = "暫不安裝",
    encoding = "本地編碼",
    encoding_desc = "設定讀取非 UTF-8 編碼文字檔案時使用的編碼",
    encoding_hint = "每行一個編碼",
    encoding_hint_placeholder = "輸入編碼，每行一個",
    tab_exclude = "排除清單",
    tab_language = "語言",
    tab_encoding = "編碼",
    tab_context_menu = "關聯選單",
    context_menu = "關聯選單",
    context_menu_desc = "在資料匣和檔案上加入'打包'，在 docx 上加入'解包'",
    tab_about = "關於",
    exclude_desc = "符合這些 glob 規則的檔案將被排除在打包之外",
    exclude_hint = "每行一個規則",
    exclude_hint_placeholder = "輸入排除模式，每行一個",
    language_desc = "變更應用程式的顯示語言",
    language_hint = "選擇顯示語言",
    close = "關閉",
    save = "儲存",
    cancel = "取消",
    scanning = "正在掃描...",
    pack_done = "完成：{count} 個檔案 -> {path}",
    ok = "確定",
    no_source_paths = "未選擇源路徑",
    extracted_details = "已解壓 {count} 個檔案到 {path}",
    context_menu_installed = "右鍵選單已安裝",
    context_menu_uninstalled = "右鍵選單已解除安裝",
};

impl I18n {
    pub fn new() -> Self {
        I18n {
            current_lang: "en".into(),
            translations: EN.iter().copied().collect(),
        }
    }

    pub fn load_lang(&mut self, lang: &str) {
        self.current_lang = lang.into();
        self.translations = match lang {
            "zh-CN" => ZH_CN.iter().copied().collect(),
            "zh-TW" => ZH_TW.iter().copied().collect(),
            _ => EN.iter().copied().collect(),
        };
    }

    pub fn get(&self, key: &str) -> String {
        let result = self.translations.get(key).copied().unwrap_or(key);
        if cfg!(debug_assertions) && !self.translations.contains_key(key) {
            eprintln!("[lang] Missing translation key: {}", key);
        }
        result.to_string()
    }

    pub fn set_lang(&mut self, lang: &str) {
        self.load_lang(lang);
    }

    pub fn current_code(&self) -> &str {
        &self.current_lang
    }

    pub fn available_langs(&self) -> Vec<(String, String)> {
        vec![
            ("en".into(), "English".into()),
            ("zh-CN".into(), "简体中文".into()),
            ("zh-TW".into(), "繁體中文".into()),
        ]
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}
