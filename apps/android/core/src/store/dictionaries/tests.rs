//! 词库页的导入、列出与删除。

use super::*;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("qingjian-android-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn import_list_and_remove() {
    let data = temp_dir("dict-data");
    let user = temp_dir("dict-user");
    let source = temp_dir("dict-source").join("food.tsv");
    std::fs::write(&source, "螺蛳粉\tluo si fen\t100\n肠粉\tchang fen\t80\n").unwrap();

    let request = DictionaryRequest {
        import: Some(source.display().to_string()),
        remove: None,
    };
    let page = dictionaries(&data, &user, &request);
    assert_eq!(page.message, "已导入「food」，共 2 条");
    assert_eq!(page.imported.len(), 1);
    assert_eq!(page.imported[0].id, "food");
    assert!(page.domains.is_empty());

    let request = DictionaryRequest {
        import: None,
        remove: Some("food".to_owned()),
    };
    let page = dictionaries(&data, &user, &request);
    assert_eq!(page.message, "已删除");
    assert!(page.imported.is_empty());
}

#[test]
fn import_rejects_files_without_entries() {
    let user = temp_dir("dict-bad");
    let source = user.join("notes.txt");
    std::fs::write(&source, "这不是词库\n").unwrap();
    let message = import_one(&source, &user);
    assert!(message.starts_with("导入失败"), "{message}");
    assert!(files(&imported_dir(&user)).is_empty());
}
