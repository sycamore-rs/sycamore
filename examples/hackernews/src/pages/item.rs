use sycamore::prelude::*;

use crate::apis::types::StoryPageData;
use crate::components::comment::CommentView;
use crate::components::show_stories::Story;

#[component]
pub async fn Item(id: i64) -> View {
    let data = crate::apis::get_story(id).await;
    match data {
        Ok(story) => {
            let StoryPageData { item, comments } = story;

            let has_comments = !comments.is_empty();
            let comments_view = View::from(
                comments
                    .into_iter()
                    .map(|comment| {
                        view! {
                            CommentView(comment=comment)
                        }
                    })
                    .collect::<Vec<View>>(),
            );
            view! {
                ul(class="list-none mb-2") { Story(story=item) }
                ul(class="list-none") {
                    if has_comments {
                        (comments_view)
                    } else {
                        "No comments yet."
                    }
                }
            }
        }
        Err(_) => view! {
            "Error fetching item."
        },
    }
}
