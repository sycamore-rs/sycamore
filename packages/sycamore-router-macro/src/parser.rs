use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_till};
use nom::combinator::{map, recognize, verify};
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, pair};
use nom::IResult;

#[derive(Debug, Clone)]
pub enum SegmentAst {
    Param(String),
    DynParam(String),
    DynSegments(String),
}

#[derive(Debug)]
pub struct RoutePathAst {
    pub(crate) segments: Vec<SegmentAst>,
}

impl RoutePathAst {
    pub fn dyn_segments(&self) -> Vec<SegmentAst> {
        self.segments
            .iter()
            .filter(|x| matches!(x, SegmentAst::DynParam(_) | &SegmentAst::DynSegments(_)))
            .cloned()
            .collect()
    }
}

fn param(i: &str) -> IResult<&str, &str> {
    take_till(|c| c == '/')(i)
}

pub fn ident_start(s: &str) -> IResult<&str, &str> {
    verify(take(1usize), |c: &str| {
        let c = c.chars().next().unwrap();
        c == '_' || unicode_xid::UnicodeXID::is_xid_start(c)
    })(s)
}

pub fn ident_continue(s: &str) -> IResult<&str, &str> {
    verify(take(1usize), |c: &str| {
        unicode_xid::UnicodeXID::is_xid_continue(c.chars().next().unwrap())
    })(s)
}

/// Parse a Rust identifier. Reference: https://doc.rust-lang.org/reference/identifiers.html
fn ident(i: &str) -> IResult<&str, &str> {
    recognize(pair(ident_start, many0(ident_continue)))(i)
}

fn dyn_param(i: &str) -> IResult<&str, &str> {
    delimited(tag("<"), ident, tag(">"))(i)
}

fn dyn_segments(i: &str) -> IResult<&str, &str> {
    delimited(tag("<"), ident, tag("..>"))(i)
}

fn segment(i: &str) -> IResult<&str, SegmentAst> {
    alt((
        map(dyn_segments, |s| SegmentAst::DynSegments(s.to_string())),
        map(dyn_param, |s| SegmentAst::DynParam(s.to_string())),
        map(param, |s| SegmentAst::Param(s.to_string())),
    ))(i)
}

pub fn route(i: &str) -> IResult<&str, RoutePathAst> {
    map(separated_list0(tag("/"), segment), |segments| {
        let segments = segments
            .into_iter()
            .filter(|x| !matches!(x, SegmentAst::Param(param) if param.is_empty()))
            .collect();
        RoutePathAst { segments }
    })(i)
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};

    use super::*;

    fn check(input: &str, expect: Expect) {
        let actual = format!("{:#?}", route(input).unwrap());
        expect.assert_eq(&actual);
    }

    #[test]
    fn index_route() {
        check(
            "/",
            expect![[r#"
                (
                    "",
                    RoutePathAst {
                        segments: [],
                    },
                )"#]],
        );
    }

    #[test]
    fn static_route() {
        check(
            "/my/static/path",
            expect![[r#"
                (
                    "",
                    RoutePathAst {
                        segments: [
                            Param(
                                "my",
                            ),
                            Param(
                                "static",
                            ),
                            Param(
                                "path",
                            ),
                        ],
                    },
                )"#]],
        );
    }

    #[test]
    fn route_with_trailing_slash() {
        check(
            "/path/",
            expect![[r#"
                (
                    "",
                    RoutePathAst {
                        segments: [
                            Param(
                                "path",
                            ),
                        ],
                    },
                )"#]],
        );
    }

    #[test]
    fn route_with_double_slashes() {
        check(
            "//path///segments////",
            expect![[r#"
                (
                    "",
                    RoutePathAst {
                        segments: [
                            Param(
                                "path",
                            ),
                            Param(
                                "segments",
                            ),
                        ],
                    },
                )"#]],
        );
    }

    #[test]
    fn route_with_no_leading_slash() {
        check(
            "my/static/path",
            expect![[r#"
                (
                    "",
                    RoutePathAst {
                        segments: [
                            Param(
                                "my",
                            ),
                            Param(
                                "static",
                            ),
                            Param(
                                "path",
                            ),
                        ],
                    },
                )"#]],
        );
    }

    #[test]
    fn route_with_no_slash() {
        check(
            "path",
            expect![[r#"
                (
                    "",
                    RoutePathAst {
                        segments: [
                            Param(
                                "path",
                            ),
                        ],
                    },
                )"#]],
        );
    }

    #[test]
    fn dyn_param() {
        check(
            "/id/<id>",
            expect![[r#"
                (
                    "",
                    RoutePathAst {
                        segments: [
                            Param(
                                "id",
                            ),
                            DynParam(
                                "id",
                            ),
                        ],
                    },
                )"#]],
        );
    }

    #[test]
    fn unnamed_dyn_param() {
        check(
            "/id/<_>",
            expect![[r#"
                (
                    "",
                    RoutePathAst {
                        segments: [
                            Param(
                                "id",
                            ),
                            DynParam(
                                "_",
                            ),
                        ],
                    },
                )"#]],
        );
    }

    #[test]
    fn dyn_segments() {
        check(
            "/page/<path..>",
            expect![[r#"
                (
                    "",
                    RoutePathAst {
                        segments: [
                            Param(
                                "page",
                            ),
                            DynSegments(
                                "path",
                            ),
                        ],
                    },
                )"#]],
        );
    }

    #[test]
    fn dyn_should_eat_slash_character() {
        check(
            "/<a/b>/",
            expect![[r#"
                (
                    "",
                    RoutePathAst {
                        segments: [
                            Param(
                                "<a",
                            ),
                            Param(
                                "b>",
                            ),
                        ],
                    },
                )"#]],
        );
    }

    #[test]
    fn dyn_param_before_dyn_segment() {
        check(
            "/<param>/<segments..>",
            expect![[r#"
                (
                    "",
                    RoutePathAst {
                        segments: [
                            DynParam(
                                "param",
                            ),
                            DynSegments(
                                "segments",
                            ),
                        ],
                    },
                )"#]],
        );
    }
}
