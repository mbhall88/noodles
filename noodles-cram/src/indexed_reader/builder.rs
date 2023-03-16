use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io,
    path::{Path, PathBuf},
};

use super::IndexedReader;
use crate::crai;

/// An indexed BAM reader builder.
#[derive(Default)]
pub struct Builder {
    index: Option<crai::Index>,
}

impl Builder {
    /// Sets an index.
    pub fn set_index(mut self, index: crai::Index) -> Self {
        self.index = Some(index);
        self
    }

    /// Builds an indexed CRAM reader from a path.
    pub fn build_from_path<P>(self, src: P) -> io::Result<IndexedReader<File>>
    where
        P: AsRef<Path>,
    {
        let src = src.as_ref();

        let index = match self.index {
            Some(index) => index,
            None => {
                let index_src = build_index_src(src);
                crai::read(index_src)?
            }
        };

        let file = File::open(src)?;

        Ok(IndexedReader::new(file, index))
    }
}

fn build_index_src<P>(src: P) -> PathBuf
where
    P: AsRef<Path>,
{
    const EXT: &str = "crai";
    push_ext(src.as_ref().into(), EXT)
}

fn push_ext<S>(path: PathBuf, ext: S) -> PathBuf
where
    S: AsRef<OsStr>,
{
    let mut s = OsString::from(path);
    s.push(".");
    s.push(ext);
    PathBuf::from(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_index_src() {
        assert_eq!(
            build_index_src("sample.cram"),
            PathBuf::from("sample.cram.crai")
        );
    }
}
