use std::cmp::Ordering;
use std::path::Path;

use log::info;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    StripPrefix(std::path::StripPrefixError),
    WalkDir(walkdir::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(inner) => write!(f, "I/O error: {inner}"),
            Error::StripPrefix(inner) => write!(f, "Strip prefix error: {inner}"),
            Error::WalkDir(inner) => write!(f, "Walk dir error: {inner}"),
        }
    }
}

impl Error {
    pub fn kind(&self) -> Option<std::io::ErrorKind> {
        match self {
            Error::Io(e) => Some(e.kind()),
            _ => None,
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::StripPrefix(e) => Some(e),
            Error::WalkDir(e) => Some(e),
        }
    }
}

/// Check if two directories are different; returns true if they differ
/// source: https://github.com/assert-rs/dir-diff/blob/master/src/lib.rs
pub fn is_dissimilar_dirs<P: AsRef<Path>, Q: AsRef<Path>>(p: P, q: Q) -> Result<bool, Error> {
    let mut a_walker = walk_dir(p)?;
    let mut b_walker = walk_dir(q)?;

    for (a, b) in (&mut a_walker).zip(&mut b_walker) {
        let a = a?;
        let b = b?;

        if a.depth() != b.depth()
            || a.file_type() != b.file_type()
            || a.file_name() != b.file_name()
            || (a.file_type().is_file() && std::fs::read(a.path())? != std::fs::read(b.path())?)
        {
            info!("Difference found at {:?} and {:?}", a.path(), b.path());
            if a.file_type().is_file() && b.file_type().is_file() {
                let a_content = std::fs::read(a.path())?;
                let b_content = std::fs::read(b.path())?;
                if a_content != b_content {
                    info!(
                        "File contents differ: {} bytes vs {} bytes",
                        a_content.len(),
                        b_content.len()
                    );
                }
            } else {
                info!("File types or names differ");
            }
            return Ok(true);
        }
    }

    Ok(a_walker.next().is_some() || b_walker.next().is_some())
}

fn walk_dir<P: AsRef<Path>>(path: P) -> Result<walkdir::IntoIter, std::io::Error> {
    let mut walkdir = WalkDir::new(path).sort_by(compare_by_file_name).into_iter();
    if let Some(Err(e)) = walkdir.next() {
        Err(e.into())
    } else {
        Ok(walkdir)
    }
}

fn compare_by_file_name(a: &DirEntry, b: &DirEntry) -> Ordering {
    a.file_name().cmp(b.file_name())
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<std::path::StripPrefixError> for Error {
    fn from(e: std::path::StripPrefixError) -> Error {
        Error::StripPrefix(e)
    }
}

impl From<walkdir::Error> for Error {
    fn from(e: walkdir::Error) -> Error {
        Error::WalkDir(e)
    }
}

/// Atomic copy directory from source to target
#[allow(dead_code)]
pub fn atomic_copy_dir<P: AsRef<Path>, Q: AsRef<Path>>(
    p: P,
    q: Q,
) -> Result<(), fs_extra::error::Error> {
    let p = p.as_ref();
    let q = q.as_ref();

    if !p.exists() {
        return Err(fs_extra::error::Error::new(
            fs_extra::error::ErrorKind::NotFound,
            "Source directory not found",
        ));
    }

    fs_extra::dir::get_dir_content(p)?;
    fs_extra::dir::create_all(q, true)?;
    fs_extra::dir::copy(p, q, &fs_extra::dir::CopyOptions::new().content_only(true))?;

    if let Ok(true) = is_dissimilar_dirs(p, q) {
        return Err(fs_extra::error::Error::new(
            fs_extra::error::ErrorKind::Other,
            "Source and destination differ",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        use std::io::ErrorKind;

        assert_eq!(
            format!(
                "{}",
                Error::Io(std::io::Error::new(ErrorKind::Other, "oh no!"))
            ),
            "I/O error: oh no!"
        );
    }

    #[test]
    fn test_is_dissimilar_dirs() {
        let dir1 = tempfile::tempdir().unwrap();
        let dir2 = tempfile::tempdir().unwrap();

        std::fs::write(dir1.path().join("file1.txt"), b"Hello").unwrap();
        std::fs::write(dir2.path().join("file1.txt"), b"Hello").unwrap();

        assert_eq!(is_dissimilar_dirs(dir1.path(), dir2.path()).unwrap(), false);

        std::fs::write(dir2.path().join("file2.txt"), b"World").unwrap();

        assert_eq!(is_dissimilar_dirs(dir1.path(), dir2.path()).unwrap(), true);

        std::fs::write(dir1.path().join("file2.txt"), b"World!").unwrap();

        assert_eq!(is_dissimilar_dirs(dir1.path(), dir2.path()).unwrap(), true);
    }

    #[test]
    fn test_atomic_copy_dir() {
        env_logger::builder().is_test(true).try_init().ok();
        let dir1 = tempfile::tempdir().unwrap();
        let dir2 = tempfile::tempdir().unwrap();
        std::fs::write(dir1.path().join("file1.txt"), b"Hello").unwrap();
        atomic_copy_dir(dir1.path(), dir2.path()).unwrap();
        assert_eq!(
            std::fs::read(dir2.path().join("file1.txt")).unwrap(),
            b"Hello"
        );
    }

    #[test]
    fn test_atomic_copy_dir_nested() {
        let dir1 = tempfile::tempdir().unwrap();
        let dir2 = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir1.path().join("subdir")).unwrap();
        std::fs::write(dir1.path().join("subdir").join("file1.txt"), b"Hello").unwrap();
        atomic_copy_dir(dir1.path(), dir2.path()).unwrap();
        assert_eq!(
            std::fs::read(dir2.path().join("subdir").join("file1.txt")).unwrap(),
            b"Hello"
        );
        assert_eq!(is_dissimilar_dirs(dir1.path(), dir2.path()).unwrap(), false);
    }

    #[test]
    fn test_atomic_copy_dir_empty_source() {
        let dir1 = tempfile::tempdir().unwrap();
        let dir2 = tempfile::tempdir().unwrap();
        atomic_copy_dir(dir1.path(), dir2.path()).unwrap();
        assert!(std::fs::read_dir(dir2.path()).unwrap().next().is_none());
    }
}
