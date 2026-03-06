use std::path::PathBuf;

pub fn db_path() -> PathBuf {
    let home = dirs::home_dir().expect("could not determine home directory");
    let dir = home.join(".sup");
    std::fs::create_dir_all(&dir).ok();
    dir.join("sup.db")
}
