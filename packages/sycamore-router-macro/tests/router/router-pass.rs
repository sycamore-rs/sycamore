use sycamore_router::Route;

#[derive(Route)]
enum Routes1 {
    #[not_found]
    NotFound,
}

#[derive(Route)]
enum Routes2 {
    #[to("/")]
    Home,
    #[to("/about")]
    About,
    #[not_found]
    NotFound,
}

#[derive(Route)]
enum Routes3 {
    #[to("/hello/<name>/<age>")]
    Hello(String, u32),
    #[to("/account/<id>")]
    Account(u32),
    #[not_found]
    NotFound,
}

#[derive(Route)]
enum Routes4 {
    #[to("/hello/<name>/<age>")]
    Hello { name: String, age: u32 },
    #[to("/account/<id>")]
    Account { id: u32 },
    #[not_found]
    NotFound,
}

#[derive(Route)]
enum Routes5 {
    #[to("/account")]
    #[preload(|_| async { todo!() })]
    Account { data: String },
    #[to("/account2")]
    #[preload(|_| async { todo!() })]
    Account2(String),
    #[to("/account3/<id>")]
    #[preload(|_| async { todo!() })]
    Account3 { id: u32, data: String },
    #[to("/account4/<id>")]
    #[preload(|_| async { todo!() })]
    Account4(u32, String),
    #[not_found]
    NotFound,
}

fn main() {}
