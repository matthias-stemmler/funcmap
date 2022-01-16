use funcmap::Equals;

enum T1 {}
enum T2 {}

impl Equals<T2> for T1 {}

fn main() {}