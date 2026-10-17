use subtitle_service::overrides::{Tag, Token, parse_block, strip_overrides, tokenize};

fn tag(name: &str, arg: &str) -> Tag {
    Tag {
        name: name.into(),
        arg: arg.into(),
    }
}

#[test]
fn longest_tag_name_wins_so_fscx_is_not_read_as_fs() {
    assert_eq!(
        parse_block(r"\fscx120\fs40\bord3\b1\blur2\be1"),
        vec![tag("fscx", "120"), tag("fs", "40"), tag("bord", "3"), tag("b", "1"), tag("blur", "2"), tag("be", "1")]
    );
}

#[test]
fn backslashes_inside_transform_parentheses_do_not_split_the_tag() {
    assert_eq!(
        parse_block(r"\t(0,500,\frz30\c&HFF&)\an8"),
        vec![tag("t", r"(0,500,\frz30\c&HFF&)"), tag("an", "8")]
    );
}

#[test]
fn unknown_tags_are_labelled_so_the_report_can_count_them() {
    assert_eq!(parse_block(r"\zzz5"), vec![tag("unknown:zzz5", "")]);
}

#[test]
fn comment_blocks_without_tags_vanish() {
    assert_eq!(tokenize("a{note to typesetter}b", 0), vec![Token::Text("ab".into())]);
}

#[test]
fn hard_and_soft_breaks_follow_wrap_style() {
    assert_eq!(strip_overrides(r"one\Ntwo\nthree\hfour", 0), "one\ntwo three\u{a0}four");
    assert_eq!(strip_overrides(r"one\ntwo", 2), "one\ntwo", "WrapStyle 2 makes \\n a hard break");
}

#[test]
fn stripping_removes_tags_and_vector_drawings_but_keeps_spoken_text() {
    let input = r"{\an7\pos(10,10)\p1}m 0 0 l 10 0 10 10{\p0}{\i1}Hi{\i0} there";
    assert_eq!(strip_overrides(input, 0), "Hi there", "drawing commands are not dialogue");
}

#[test]
fn unclosed_brace_is_kept_as_literal_text() {
    assert_eq!(strip_overrides("price {in brackets", 0), "price {in brackets");
}
