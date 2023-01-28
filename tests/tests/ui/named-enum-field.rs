use tsap::param;

#[param]
enum Param<const C: bool> {
    NotSupported {
        value: f32,
    }
}

fn main() {}
