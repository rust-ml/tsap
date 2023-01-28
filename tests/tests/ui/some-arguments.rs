use tsap::param;

#[param(should_not_have = "this")]
struct Param<const C: bool> {
    no_const: u32
}

fn main() {}
