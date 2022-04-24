/// Usage of [`FuncMap`] to "deeply" convert from relative to absolute paths
/// using newtype wrappers
use funcmap::FuncMap;
use paths::*;

/// Example data structure illustrating the use of [`FuncMap`]
/// `T` is meant to be either [`RelativePath<P>`] or [`AbsolutePath<P>`] where
/// `P: AsRef<Path>`
#[derive(FuncMap, Debug)]
struct FilePaths<T> {
    manifest_path: T,
    src_paths: Vec<T>,
}

fn main() {
    let relative_paths = FilePaths::<RelativePath<_>> {
        manifest_path: "Cargo.toml".into(),
        src_paths: vec!["src/lib.rs".into(), "src/main.rs".into()],
    };

    let base_path = "my_project";
    let absolute_paths = relative_paths.func_map(|rel_path| rel_path.to_absolute(base_path));

    println!("{:?}", absolute_paths);
}

/// Helpers dealing with absolute and relative paths
mod paths {
    use std::path::{Path, PathBuf};

    #[derive(Debug)]
    pub struct AbsolutePath<P>(P);

    #[derive(Debug)]
    pub struct RelativePath<P>(P);

    impl<P> From<P> for RelativePath<P> {
        fn from(path: P) -> Self {
            Self(path)
        }
    }

    impl<P> RelativePath<P>
    where
        P: AsRef<Path>,
    {
        pub fn to_absolute<Q>(&self, base_path: Q) -> AbsolutePath<PathBuf>
        where
            Q: AsRef<Path>,
        {
            AbsolutePath(base_path.as_ref().join(&self.0))
        }
    }
}
