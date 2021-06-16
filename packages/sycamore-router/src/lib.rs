//! The Sycamore Router.

pub trait Router {
    fn match_route(path: &[&str]) -> Self;
}

pub enum Segment {
    /// Match a specific segment.
    Param(String),
    /// Match an arbitrary segment that is captured.
    DynParam,
    /// Match an arbitrary amount of segments that are captured.
    DynSegments,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Capture<'a> {
    DynParam(&'a str),
    DynSegments(Vec<&'a str>),
}

pub struct Route {
    segments: Vec<Segment>,
}

impl Route {
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

#[cfg(test)]
mod tests {
    use super::*;
    use Segment::*;

    fn check(path: &str, route: Route, expected: Option<Vec<Capture>>) {
        let path = path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        assert_eq!(route.match_path(&path), expected);
    }

    #[test]
    fn index_path() {
        check("/", Route::new(Vec::new()), Some(Vec::new()));
    }

    #[test]
    fn static_path_single_segment() {
        check(
            "/path",
            Route::new(vec![Param("path".to_string())]),
            Some(Vec::new()),
        );
    }

    #[test]
    fn static_path_multiple_segments() {
        check(
            "/my/static/path",
            Route::new(vec![
                Param("my".to_string()),
                Param("static".to_string()),
                Param("path".to_string()),
            ]),
            Some(Vec::new()),
        );
    }

    #[test]
    fn do_not_match_if_leftover_segments() {
        check("/path", Route::new(vec![]), None);
        check(
            "/my/static/path",
            Route::new(vec![Param("my".to_string()), Param("static".to_string())]),
            None,
        );
    }

    #[test]
    fn dyn_param_single_segment() {
        check(
            "/abcdef",
            Route::new(vec![DynParam]),
            Some(vec![Capture::DynParam("abcdef")]),
        );
    }

    #[test]
    fn dyn_param_with_leading_segment() {
        check(
            "/id/abcdef",
            Route::new(vec![Param("id".to_string()), DynParam]),
            Some(vec![Capture::DynParam("abcdef")]),
        );
    }

    #[test]
    fn dyn_param_with_leading_and_trailing_segment() {
        check(
            "/id/abcdef/account",
            Route::new(vec![
                Param("id".to_string()),
                DynParam,
                Param("account".to_string()),
            ]),
            Some(vec![Capture::DynParam("abcdef")]),
        );
    }

    #[test]
    fn dyn_param_final_missing_root() {
        check("/", Route::new(vec![DynParam]), None);
    }

    #[test]
    fn dyn_param_final_missing() {
        check(
            "/id",
            Route::new(vec![Param("id".to_string()), DynParam]),
            None,
        );
    }

    #[test]
    fn multiple_dyn_params() {
        check(
            "/a/b",
            Route::new(vec![DynParam, DynParam]),
            Some(vec![Capture::DynParam("a"), Capture::DynParam("b")]),
        );
    }

    #[test]
    fn dyn_segments_at_root() {
        check(
            "/a/b/c",
            Route::new(vec![DynSegments]),
            Some(vec![Capture::DynSegments(vec!["a", "b", "c"])]),
        );
    }

    #[test]
    fn dyn_segments_final() {
        check(
            "/id/a/b/c",
            Route::new(vec![Param("id".to_string()), DynSegments]),
            Some(vec![Capture::DynSegments(vec!["a", "b", "c"])]),
        );
    }

    #[test]
    fn dyn_segments_capture_lazy() {
        check(
            "/id/a/b/c/end",
            Route::new(vec![
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
            Route::new(vec![DynSegments]),
            Some(vec![Capture::DynSegments(Vec::new())]),
        );
    }

    #[test]
    fn multiple_dyn_segments() {
        check(
            "/a/b/c/param/e/f/g",
            Route::new(vec![DynSegments, Param("param".to_string()), DynSegments]),
            Some(vec![
                Capture::DynSegments(vec!["a", "b", "c"]),
                Capture::DynSegments(vec!["e", "f", "g"]),
            ]),
        );
    }
}
