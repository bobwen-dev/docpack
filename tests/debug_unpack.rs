fn main() {
    let dir = tempfile::tempdir().unwrap();
    let dir_path = dir.path();
    
    println!("Temp dir: {}", dir_path.display());
    
    // Create test files
    std::fs::create_dir(dir_path.join("src")).unwrap();
    std::fs::write(dir_path.join("src/main.rs"), "fn main() {}").unwrap();
    std::fs::write(dir_path.join("readme.md"), "# DocPack").unwrap();
    
    // Pack
    let docx_path = dir_path.join("output.docx");
    let rules = docpack::ignore::ExcludeRules::new();
    let result = docpack::pack::pack_dir(dir_path, &docx_path, &rules, &[], None);
    match result {
        Ok(res) => println!("Packed {} files", res.file_count),
        Err(e) => {
            eprintln!("Pack error: {}", e);
            return;
        }
    }
    
    // Unpack
    let unpack_dir = dir_path.join("unpacked");
    let result = docpack::unpack::unpack_docx(&docx_path, &unpack_dir);
    match result {
        Ok(res) => println!("Unpacked {} files to {}", res.file_count, res.output_dir.display()),
        Err(e) => {
            eprintln!("Unpack error: {}", e);
            return;
        }
    }
    
    // List files in unpacked directory
    println!("\nFiles in unpacked directory:");
    for entry in walkdir::WalkDir::new(&unpack_dir) {
        match entry {
            Ok(e) => println!("  {}", e.path().display()),
            Err(e) => eprintln!("  Error: {}", e),
        }
    }
    
    let main_rs = unpack_dir.join("src/main.rs");
    println!("\nChecking: {}", main_rs.display());
    println!("Exists: {}", main_rs.exists());
    
    if let Some(parent) = main_rs.parent() {
        println!("Parent exists: {}", parent.exists());
    }
}
