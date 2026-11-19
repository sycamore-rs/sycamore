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
        li(class="py-3") {
            div(class="mb-2 flex flex-wrap gap-x-3 gap-y-1 text-sm text-slate-600") {
                a(href=by_url, class="font-medium text-emerald-800 hover:underline") { (by) }
                span { (time.format("%D %l:%M %p").to_string()) }
            }
            div(class="break-words leading-relaxed [&_p]:mt-2 [&_pre]:overflow-x-auto [&_a]:underline", dangerously_set_inner_html=text)
            ul(class="mt-2 list-none border-l border-slate-200 pl-2 empty:hidden sm:pl-4") { (sub_comments_view) }
        }
    }
}
