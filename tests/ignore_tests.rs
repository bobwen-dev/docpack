use docpack::ignore::ExcludeRules;

#[test]
fn test_default_excludes() {
    let rules = ExcludeRules::new();
    assert!(rules.is_excluded(".git/config"));
    assert!(rules.is_excluded("node_modules/foo"));
    assert!(rules.is_excluded("node_modules"));
    assert!(rules.is_excluded("target/debug/app.exe"));
    assert!(rules.is_excluded("image.png"));
    assert!(!rules.is_excluded("src/main.rs"));
    assert!(!rules.is_excluded("readme.md"));
}

#[test]
fn test_custom_pattern() {
    let mut rules = ExcludeRules::new();
    rules.add_pattern("*.log");
    assert!(rules.is_excluded("error.log"));
    assert!(!rules.is_excluded("error.txt"));
}

#[test]
fn test_dir_pattern() {
    let mut rules = ExcludeRules::new();
    rules.add_pattern("build/");
    assert!(rules.is_excluded("build/output.o"));
    assert!(rules.is_excluded("build/sub/dir/file.o"));
    assert!(!rules.is_excluded("src/build.rs"));
}

#[test]
fn test_glob_starstar() {
    let mut rules = ExcludeRules::new();
    rules.add_pattern("**/temp");
    assert!(rules.is_excluded("a/b/temp"));
    assert!(rules.is_excluded("temp"));
}

#[test]
fn test_glob_starstar_middle() {
    let mut rules = ExcludeRules::new();
    rules.add_pattern("a/**/b");
    assert!(rules.is_excluded("a/b"));
    assert!(rules.is_excluded("a/x/b"));
    assert!(rules.is_excluded("a/x/y/b"));
    assert!(rules.is_excluded("a/xc/b"));
    assert!(!rules.is_excluded("x/a/b"));
}

#[test]
fn test_glob_starstar_trailing() {
    let mut rules = ExcludeRules::new();
    rules.add_pattern("a/**");
    assert!(rules.is_excluded("a/b"));
    assert!(rules.is_excluded("a/b/c"));
    assert!(!rules.is_excluded("a"));
    assert!(!rules.is_excluded("b/a"));
}

#[test]
fn test_glob_starstar_alone() {
    let mut rules = ExcludeRules::new();
    rules.add_pattern("**");
    assert!(rules.is_excluded("anything"));
    assert!(rules.is_excluded("a/b/c"));
}

#[test]
fn test_glob_starstar_deep_nested() {
    let mut rules = ExcludeRules::new();
    rules.add_pattern("x/**/z");
    assert!(rules.is_excluded("x/z"));
    assert!(rules.is_excluded("x/y/z"));
    assert!(rules.is_excluded("x/y/y/z"));
    assert!(!rules.is_excluded("x/y/zz"));
    assert!(!rules.is_excluded("xx/y/z"));
}

#[test]
fn test_glob_starstar_multiple() {
    let mut rules = ExcludeRules::new();
    rules.add_pattern("a/**/b/**/c");
    assert!(rules.is_excluded("a/b/c"));
    assert!(rules.is_excluded("a/x/b/y/c"));
    assert!(rules.is_excluded("a/x/y/b/z/w/c"));
    assert!(!rules.is_excluded("a/x/b/y/d"));
}

#[test]
fn test_glob_question() {
    let mut rules = ExcludeRules::new();
    rules.add_pattern("?.txt");
    assert!(rules.is_excluded("a.txt"));
    assert!(!rules.is_excluded("ab.txt"));
}

#[test]
fn test_basename_matching() {
    let mut rules = ExcludeRules::new();
    rules.add_pattern("config");
    assert!(rules.is_excluded("config"));
    assert!(rules.is_excluded("dir/config"));
    assert!(!rules.is_excluded("config.json"));
}

#[test]
fn test_negation() {
    let mut rules = ExcludeRules::new();
    rules.add_pattern("*.log");
    rules.add_pattern("!keep.log");
    assert!(rules.is_excluded("error.log"));
    assert!(!rules.is_excluded("keep.log"));
    assert!(!rules.is_excluded("sub/keep.log"));
}

#[test]
fn test_anchored_pattern() {
    let mut rules = ExcludeRules {
        patterns: Vec::new(),
    };
    rules.add_pattern("/build");
    assert!(rules.is_excluded("build"));
    assert!(!rules.is_excluded("src/build"));
}

#[test]
fn test_directory_only() {
    let mut rules = ExcludeRules::new();
    rules.add_pattern("logs/");
    assert!(rules.is_excluded("logs/app.log"));
    assert!(rules.is_excluded("logs"));
    assert!(!rules.is_excluded("logs.txt"));
}

#[test]
fn test_directory_only_fresh() {
    let mut rules = ExcludeRules {
        patterns: Vec::new(),
    };
    rules.add_pattern("logs/");
    assert!(rules.is_excluded("logs/app.log"));
    assert!(rules.is_excluded("logs"));
    assert!(!rules.is_excluded("logs.txt"));
}

#[test]
fn test_escaped_hash() {
    let mut rules = ExcludeRules::new();
    rules.add_pattern("#\\!secret");
    assert!(rules.is_excluded("#!secret"));
}

#[test]
fn test_comment_lines() {
    let mut rules = ExcludeRules::new();
    rules.add_pattern("# this is not a comment");
    assert!(rules.is_excluded("# this is not a comment"));
}

#[test]
fn test_comment_lines_from_load() {
    let tmp = std::env::temp_dir().join("test_comment.txt");
    std::fs::write(&tmp, "# this is a comment\n#commented\n").unwrap();
    let rules = ExcludeRules::load(&tmp);
    let pattern_strings: Vec<&str> = rules.patterns.iter().map(|p| p.pattern.as_str()).collect();
    assert!(!pattern_strings.contains(&"# this is not a comment"));
    assert!(!pattern_strings.contains(&"#commented"));
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn test_load_from_file() {
    let tmp = std::env::temp_dir().join("test_docpackignore.txt");
    std::fs::write(&tmp, "build/\n*.log\n!keep.log\n").unwrap();
    let rules = ExcludeRules::load(&tmp);
    assert!(rules.is_excluded("build/output.o"));
    assert!(rules.is_excluded("error.log"));
    assert!(!rules.is_excluded("keep.log"));
    assert!(!rules.is_excluded("sub/keep.log"));
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn test_from_rules_text_no_defaults() {
    let rules = ExcludeRules::from_rules_text("*.log\n*.tmp\n");
    assert!(rules.is_excluded("error.log"));
    assert!(rules.is_excluded("file.tmp"));
    assert!(!rules.is_excluded("node_modules/foo"));
    assert!(!rules.is_excluded("image.png"));
}
