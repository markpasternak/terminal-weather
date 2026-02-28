use std::fs;
use std::os::unix::fs::DirBuilderExt;

fn main() {
    let parent = "test_dir/nested";
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(parent)
        .unwrap();
    let metadata = fs::metadata(parent).unwrap();
    use std::os::unix::fs::PermissionsExt;
    println!("{:o}", metadata.permissions().mode() & 0o777);
}
