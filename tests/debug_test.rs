use std::path::PathBuf;

fn main() {
    let tmp = std::env::temp_dir();
    println!("temp_dir: {}", tmp.display());
    
    let p = PathBuf::from("a:b.txt");
    println!("path: {}", p.display());
    println!("canonical: {:?}", p.canonicalize());
    
    let trimmed = "a:b.txt".trim();
    let normalized = trimmed.replace('\\', "/");
    println!("normalized: {}", normalized);
    
    // Check drive letter pattern
    let drive_pattern = format!("{}:", "docpack".chars().next().unwrap_or('X'));
    println!("drive_pattern: {}", drive_pattern);
    println!("starts_with drive: {}", normalized.starts_with(&drive_pattern));
    
    let first_char = normalized.chars().next();
    println!("first char: {:?}", first_char);
    if let Some(c) = first_char {
        println!("is_ascii_alphabetic: {}", c.is_ascii_alphabetic());
        let specific = format!("{}:", c);
        println!("specific: {}, starts_with: {}", specific, normalized.starts_with(&specific));
    }
}
