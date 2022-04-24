/// Usage of [`TryFuncMap`] for "deep" fallible conversions
use funcmap::TryFuncMap;

/// Example data structures illustrating the use of [`TryFuncMap`]
/// `T` is meant to be either `&str` or an integer
#[derive(TryFuncMap, Debug)]
struct Tree<T> {
    nodes: Vec<TreeNode<T>>,
}

#[derive(TryFuncMap, Debug)]
struct TreeNode<T> {
    id: T,
    parent_id: Option<T>,
}

fn main() {
    // tree using string slices as node IDs
    let tree_str = Tree {
        nodes: vec![
            TreeNode {
                id: "0",
                parent_id: None,
            },
            TreeNode {
                id: "1",
                parent_id: Some("0"),
            },
            TreeNode {
                id: "2",
                parent_id: Some("0"),
            },
        ],
    };

    // "deeply" apply a (fallible) conversion from string slices to integers
    let tree_i32: Result<Tree<i32>, _> = tree_str.try_func_map(|id| id.parse());

    println!("{:?}", tree_i32);
}
