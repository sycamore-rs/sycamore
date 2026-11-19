use sycamore::prelude::*;

use crate::apis::types::Comment;

#[component]
pub fn CommentView(comment: Comment) -> View {
    let Comment {
        id: _,
        by,
        text,
        time,
        kids: _,
        sub_comments,
        r#type: _,
    } = comment;

    let by_url = format!("user/{}", by);

    let sub_comments_view = View::from(
        sub_comments
            .into_iter()
            .map(|comment| {
                view! {
                    CommentView(comment=comment)
                }
            })
            .collect::<Vec<View>>(),
    );

    view! {
        li(class="mt-2") {
            div(class="mb-2 text-gray-600 border-t border-gray-300") {
                a(href=by_url) { (by) } " | " span { (time.format("%D %l:%M %p").to_string()) }
            }
            p(dangerously_set_inner_html=text)
            ul(class="list-none ml-5") {
                (sub_comments_view)
            }
        }
    }
}
