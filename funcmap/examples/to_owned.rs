use funcmap::FuncMap;

/// Example data structures illustrating the use of [FuncMap]
/// `T` is meant to be either `&str` or `String`
#[derive(FuncMap, Debug)]
struct Group<T> {
    name: T,
    members: Vec<Member<T>>,
}

#[derive(FuncMap, Debug)]
struct Member<T> {
    name: T,
}

fn main() {
    println!("{:?}", load_group());
}

/// Loads a [Group] from a data source, e.g. a file (here for the sake of simplicity: from a local buffer)
fn load_group() -> Group<String> {
    // local buffer (could e.g. be loaded from a file, will be dropped when `load_group` returns)
    let buffer = String::from("Example Group:Member 1,Member 2");

    // parsed data, borrows from local variable `buffer`, cannot be returned
    let group = parse_group(&buffer);

    // use `func_map` to perform a "deep" `to_owned`, creating owned data that can be returned
    group.func_map(ToOwned::to_owned)
}

/// Parses a [Group] from a string slice
fn parse_group(input: &str) -> Group<&str> {
    let [group_name, member_names]: [&str; 2] =
        input.split(':').collect::<Vec<_>>().try_into().unwrap();

    let members = member_names
        .split(',')
        .map(|name| Member { name })
        .collect();

    Group {
        name: group_name,
        members,
    }
}
