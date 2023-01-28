use tsap::param;

#[param]
enum Param<'a, const C: bool> {
    RandomForest,
    SVClassifier
}

#[param]
enum Param<T, const C: bool> {
    RandomForest(T),
    SVClassifier
}

fn main() {}
