use tsap::param;

#[param]
#[derive(Debug)]
struct SingleField<const C: bool> {
    value: f32,
}

fn main() {}
