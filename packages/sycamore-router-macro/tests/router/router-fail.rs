use sycamore_router::Route;

#[derive(Route)]
struct Routes1 {}

// Missing #[not_found]
#[derive(Route)]
enum Routes2 {}

#[derive(Route)]
enum Routes3 {
    #[not_found]
    NotFound(i32), // Cannot have field
}

#[derive(Route)]
enum Routes4 {
    #[to("<capture>")]
    Path, // Missing capture field
    #[not_found]
    NotFound,
}

#[derive(Route)]
enum Routes5 {
    #[to("<capture>")]
    Path {}, // Missing capture field
    #[not_found]
    NotFound,
}

#[derive(Route)]
enum Routes6 {
    #[to("<capture>")]
    Path { not_capture: u32 }, // Wrong capture field name
    #[not_found]
    NotFound,
}

#[derive(Route)]
enum Routes7 {
    #[to("<a>/<b>")]
    Path { b: u32, a: u32 }, // Wrong order
    #[not_found]
    NotFound,
}

#[derive(Route)]
enum Routes8 {
    #[to("<a/b>")] // `a/b` is not an identifier
    Path { a: u32 },
    #[not_found]
    NotFound,
}

#[derive(Route)]
enum Routes9 {
    #[to("/")]
    #[preload(|_| async { todo!() })]
    Path, // Missing `data` field.
    #[not_found]
    NotFound,
}

#[derive(Route)]
enum Routes10 {
    #[to("/")]
    #[preload(async { todo!() })] // Should be closure.
    Path { data: String },
    #[not_found]
    NotFound,
}

fn main() {}
