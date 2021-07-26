//! The Sycamore Router.

// Alias self to sycamore_router for proc-macros.
extern crate self as sycamore_router;

mod router;

pub use sycamore_router_macro::Route;

use std::str::FromStr;

pub use router::*;

/// Trait that is implemented for `enum`s that can match routes.
///
/// This trait should not be implemented manually. Use the [`Route`](derive@Route) derive macro
/// instead.
#[async_trait::async_trait]
pub trait Route {
    async fn match_route(path: &[&str]) -> Self;
}

/// Represents an URL segment or segments.
pub enum Segment {
    /// Match a specific segment.
    Param(String),
    /// Match an arbitrary segment that is captured.
    DynParam,
    /// Match an arbitrary amount of segments that are captured.
    DynSegments,
}

/// Represents a capture of an URL segment or segments.
#[derive(Debug, PartialEq, Eq)]
pub enum Capture<'a> {
    DynParam(&'a str),
    DynSegments(Vec<&'a str>),
}

impl<'a> Capture<'a> {
    pub fn as_dyn_param(&self) -> Option<&&'a str> {
        if let Self::DynParam(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_dyn_segments(&self) -> Option<&Vec<&'a str>> {
        if let Self::DynSegments(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

/// A list of [`Segment`]s.
pub struct RoutePath {
    segments: Vec<Segment>,
}

impl RoutePath {
    pub fn new(segments: Vec<Segment>) -> Self {
        Self { segments }
    }

    pub fn match_path<'a>(&self, path: &[&'a str]) -> Option<Vec<Capture<'a>>> {
        let mut paths = path.iter();
        let mut segments = self.segments.iter();
        let mut captures = Vec::new();

        while let Some(segment) = segments.next() {
            match segment {
                Segment::Param(param) => {
                    if paths.next() != Some(&param.as_str()) {
                        return None;
                    }
                }
                Segment::DynParam => {
                    if let Some(p) = paths.next() {
                        captures.push(Capture::DynParam(*p));
                    } else {
                        return None;
                    }
                }
                Segment::DynSegments => {
                    if let Some(next_segment) = segments.next() {
                        // Capture until match with Segment::Param.
                        match next_segment {
                            Segment::Param(next_param) => {
                                let mut capture = Vec::new();
                                while let Some(next_path) = paths.next() {
                                    if next_path == next_param {
                                        captures.push(Capture::DynSegments(capture));
                                        break;
                                    } else {
                                        capture.push(next_path);
                                    }
                                }
                            }
                            _ => unreachable!("segment following DynSegments cannot be dynamic"),
                        }
                    } else {
                        // All remaining segments are captured.
                        let mut capture = Vec::new();
                        while let Some(next_path) = paths.next() {
                            capture.push(*next_path);
                        }
                        captures.push(Capture::DynSegments(capture));
                    }
                }
            }
        }

        if paths.next().is_some() {
            return None; // Leftover segments in paths.
        }

        Some(captures)
    }
}

/// Fallible conversion between a param capture into a value.
///
/// Implemented for all types that implement [`FromStr`] by default.
pub trait FromParam {
    /// Set the value of the capture variable with the value of the `param`. Returns `false` if
    /// unsuccessful (e.g. parsing error).
    #[must_use]
    fn set_value(&mut self, param: &str) -> bool;
}

impl<T> FromParam for T
where
    T: FromStr,
{
    fn set_value(&mut self, param: &str) -> bool {
        match param.parse() {
            Ok(val) => {
                *self = val;
                true
            }
            Err(_) => false,
        }
    }
}

/// Fallible conversion between a list of param captures into a value.
pub trait FromSegments {
    /// Sets the value of the capture variable with the value of `segments`. Returns `false` if
    /// unsuccessful (e.g. parsing error).
    #[must_use]
    fn set_value(&mut self, segments: &[&str]) -> bool;
}

impl<T> FromSegments for Vec<T>
where
    T: FromParam + Default,
{
    fn set_value(&mut self, segments: &[&str]) -> bool {
        *self = Vec::with_capacity(segments.len());
        for segment in segments {
            let mut value = T::default();
            if !value.set_value(segment) {
                return false;
            }
            self.push(value);
        }
        true
    }
}

/// Re-exports for use by `sycamore-router-macro`. Not intended for use by end-users.
#[doc(hidden)]
pub mod rt {
    pub use async_trait::async_trait;
}

#[cfg(test)]
mod tests {
    use super::*;
    use Segment::*;

    fn check(path: &str, route: RoutePath, expected: Option<Vec<Capture>>) {
        let path = path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        assert_eq!(route.match_path(&path), expected);
    }

    #[test]
    fn index_path() {
        check("/", RoutePath::new(Vec::new()), Some(Vec::new()));
    }

    #[test]
    fn static_path_single_segment() {
        check(
            "/path",
            RoutePath::new(vec![Param("path".to_string())]),
            Some(Vec::new()),
        );
    }

    #[test]
    fn static_path_multiple_segments() {
        check(
            "/my/static/path",
            RoutePath::new(vec![
                Param("my".to_string()),
                Param("static".to_string()),
                Param("path".to_string()),
            ]),
            Some(Vec::new()),
        );
    }

    #[test]
    fn do_not_match_if_leftover_segments() {
        check("/path", RoutePath::new(vec![]), None);
        check(
            "/my/static/path",
            RoutePath::new(vec![Param("my".to_string()), Param("static".to_string())]),
            None,
        );
    }

    #[test]
    fn dyn_param_single_segment() {
        check(
            "/abcdef",
            RoutePath::new(vec![DynParam]),
            Some(vec![Capture::DynParam("abcdef")]),
        );
    }

    #[test]
    fn dyn_param_with_leading_segment() {
        check(
            "/id/abcdef",
            RoutePath::new(vec![Param("id".to_string()), DynParam]),
            Some(vec![Capture::DynParam("abcdef")]),
        );
    }

    #[test]
    fn dyn_param_with_leading_and_trailing_segment() {
        check(
            "/id/abcdef/account",
            RoutePath::new(vec![
                Param("id".to_string()),
                DynParam,
                Param("account".to_string()),
            ]),
            Some(vec![Capture::DynParam("abcdef")]),
        );
    }

    #[test]
    fn dyn_param_final_missing_root() {
        check("/", RoutePath::new(vec![DynParam]), None);
    }

    #[test]
    fn dyn_param_final_missing() {
        check(
            "/id",
            RoutePath::new(vec![Param("id".to_string()), DynParam]),
            None,
        );
    }

    #[test]
    fn multiple_dyn_params() {
        check(
            "/a/b",
            RoutePath::new(vec![DynParam, DynParam]),
            Some(vec![Capture::DynParam("a"), Capture::DynParam("b")]),
        );
    }

    #[test]
    fn dyn_segments_at_root() {
        check(
            "/a/b/c",
            RoutePath::new(vec![DynSegments]),
            Some(vec![Capture::DynSegments(vec!["a", "b", "c"])]),
        );
    }

    #[test]
    fn dyn_segments_final() {
        check(
            "/id/a/b/c",
            RoutePath::new(vec![Param("id".to_string()), DynSegments]),
            Some(vec![Capture::DynSegments(vec!["a", "b", "c"])]),
        );
    }

    #[test]
    fn dyn_segments_capture_lazy() {
        check(
            "/id/a/b/c/end",
            RoutePath::new(vec![
                Param("id".to_string()),
                DynSegments,
                Param("end".to_string()),
            ]),
            Some(vec![Capture::DynSegments(vec!["a", "b", "c"])]),
        );
    }

    #[test]
    fn dyn_segments_can_capture_zero_segments() {
        check(
            "/",
            RoutePath::new(vec![DynSegments]),
            Some(vec![Capture::DynSegments(Vec::new())]),
        );
    }

    #[test]
    fn multiple_dyn_segments() {
        check(
            "/a/b/c/param/e/f/g",
            RoutePath::new(vec![DynSegments, Param("param".to_string()), DynSegments]),
            Some(vec![
                Capture::DynSegments(vec!["a", "b", "c"]),
                Capture::DynSegments(vec!["e", "f", "g"]),
            ]),
        );
    }

    mod integration {
        use crate::*;

        #[test]
        fn simple_router() {
            futures::executor::block_on(async {
                #[derive(Debug, PartialEq, Eq, Route)]
                enum Routes {
                    #[to("/")]
                    Home,
                    #[to("/about")]
                    About,
                    #[not_found]
                    NotFound,
                }

                assert_eq!(Routes::match_route(&[]).await, Routes::Home);
                assert_eq!(Routes::match_route(&["about"]).await, Routes::About);
                assert_eq!(Routes::match_route(&["404"]).await, Routes::NotFound);
                assert_eq!(
                    Routes::match_route(&["about", "something"]).await,
                    Routes::NotFound
                );
            });
        }

        #[test]
        fn router_dyn_params() {
            futures::executor::block_on(async {
                #[derive(Debug, PartialEq, Eq, Route)]
                enum Routes {
                    #[to("/account/<id>")]
                    Account { id: u32 },
                    #[not_found]
                    NotFound,
                }

                assert_eq!(
                    Routes::match_route(&["account", "123"]).await,
                    Routes::Account { id: 123 }
                );
                assert_eq!(
                    Routes::match_route(&["account", "-1"]).await,
                    Routes::NotFound
                );
                assert_eq!(
                    Routes::match_route(&["account", "abc"]).await,
                    Routes::NotFound
                );
                assert_eq!(Routes::match_route(&["account"]).await, Routes::NotFound);
            });
        }

        #[test]
        fn router_multiple_dyn_params() {
            futures::executor::block_on(async {
                #[derive(Debug, PartialEq, Eq, Route)]
                enum Routes {
                    #[to("/hello/<name>/<age>")]
                    Hello { name: String, age: u32 },
                    #[not_found]
                    NotFound,
                }

                assert_eq!(
                    Routes::match_route(&["hello", "Bob", "21"]).await,
                    Routes::Hello {
                        name: "Bob".to_string(),
                        age: 21
                    }
                );
                assert_eq!(
                    Routes::match_route(&["hello", "21", "Bob"]).await,
                    Routes::NotFound
                );
                assert_eq!(Routes::match_route(&["hello"]).await, Routes::NotFound);
            });
        }

        #[test]
        fn router_multiple_dyn_segments() {
            futures::executor::block_on(async {
                #[derive(Debug, PartialEq, Eq, Route)]
                enum Routes {
                    #[to("/path/<path..>")]
                    Path { path: Vec<String> },
                    #[to("/numbers/<numbers..>")]
                    Numbers { numbers: Vec<u32> },
                    #[not_found]
                    NotFound,
                }

                assert_eq!(
                    Routes::match_route(&["path", "a", "b", "c"]).await,
                    Routes::Path {
                        path: vec!["a".to_string(), "b".to_string(), "c".to_string()]
                    }
                );
                assert_eq!(
                    Routes::match_route(&["numbers", "1", "2", "3"]).await,
                    Routes::Numbers {
                        numbers: vec![1, 2, 3]
                    }
                );
                assert_eq!(
                    Routes::match_route(&["path"]).await,
                    Routes::Path { path: Vec::new() }
                );
            });
        }

        #[test]
        fn router_multiple_dyn_segments_match_lazy() {
            futures::executor::block_on(async {
                #[derive(Debug, PartialEq, Eq, Route)]
                enum Routes {
                    #[to("/path/<path..>/end")]
                    Path { path: Vec<u32> },
                    #[not_found]
                    NotFound,
                }

                assert_eq!(
                    Routes::match_route(&["path", "1", "2", "end"]).await,
                    Routes::Path { path: vec![1, 2] }
                );
                assert_eq!(
                    Routes::match_route(&["path", "end"]).await,
                    Routes::Path { path: Vec::new() }
                );
                assert_eq!(
                    Routes::match_route(&["path", "1", "end", "2"]).await,
                    Routes::NotFound
                );
            });
        }
    }
}
