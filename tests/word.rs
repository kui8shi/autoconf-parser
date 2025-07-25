#![deny(rust_2018_idioms)]
use autotools_parser::ast::ComplexWord::*;
use autotools_parser::ast::MayM4::*;
use autotools_parser::ast::SimpleWord::*;
use autotools_parser::ast::*;
use autotools_parser::parse::ParseErrorKind::*;
use autotools_parser::token::Token;

mod parse_support;
use crate::parse_support::*;

#[test]
fn test_word_single_quote_valid() {
    let correct = single_quoted("abc&&||\n\n#comment\nabc");
    assert_eq!(
        Some(correct),
        make_parser("'abc&&||\n\n#comment\nabc'").word().unwrap()
    );
}

#[test]
fn test_word_single_quote_valid_slash_remains_literal() {
    let correct = single_quoted("\\\n");
    assert_eq!(Some(correct), make_parser("'\\\n'").word().unwrap());
}

#[test]
fn test_word_single_quote_valid_does_not_quote_single_quotes() {
    let correct = single_quoted("hello \\");
    assert_eq!(Some(correct), make_parser("'hello \\'").word().unwrap());
}

#[test]
fn test_word_single_quote_invalid_missing_close_quote() {
    assert_eq!(
        Err(Unmatched(Token::SingleQuote, src(0, 1, 1)).into()),
        make_parser("'hello").word()
    );
}

#[test]
fn test_word_double_quote_valid() {
    let correct = TopLevelWord(Single(Word::DoubleQuoted(vec![Shell(Literal(
        String::from("abc&&||\n\n#comment\nabc"),
    ))])));
    assert_eq!(
        Some(correct),
        make_parser("\"abc&&||\n\n#comment\nabc\"").word().unwrap()
    );
}

#[test]
fn test_word_double_quote_valid_recognizes_parameters() {
    let correct = TopLevelWord(Single(Word::DoubleQuoted(vec![
        Shell(Literal(String::from("test asdf"))),
        Shell(Param(Parameter::Var(String::from("foo")))),
        Shell(Literal(String::from(" $"))),
    ])));

    assert_eq!(
        Some(correct),
        make_parser("\"test asdf$foo $\"").word().unwrap()
    );
}

#[test]
fn test_word_double_quote_valid_recognizes_backticks() {
    let correct = TopLevelWord(Single(Word::DoubleQuoted(vec![
        Shell(Literal(String::from("test asdf "))),
        Shell(Subst(Box::new(ParameterSubstitution::Command(vec![cmd(
            "foo",
        )])))),
    ])));

    assert_eq!(
        Some(correct),
        make_parser("\"test asdf `foo`\"").word().unwrap()
    );
}

#[test]
fn test_word_double_quote_valid_slash_escapes_dollar() {
    let correct = TopLevelWord(Single(Word::DoubleQuoted(vec![
        Shell(Literal(String::from("test"))),
        Shell(Escaped(String::from("$"))),
        Shell(Literal(String::from("foo "))),
        Shell(Param(Parameter::At)),
    ])));

    assert_eq!(
        Some(correct),
        make_parser("\"test\\$foo $@\"").word().unwrap()
    );
}

#[test]
fn test_word_double_quote_valid_slash_escapes_backtick() {
    let correct = TopLevelWord(Single(Word::DoubleQuoted(vec![
        Shell(Literal(String::from("test"))),
        Shell(Escaped(String::from("`"))),
        Shell(Literal(String::from(" "))),
        Shell(Param(Parameter::Star)),
    ])));

    assert_eq!(Some(correct), make_parser("\"test\\` $*\"").word().unwrap());
}

#[test]
fn test_word_double_quote_valid_slash_escapes_double_quote() {
    let correct = TopLevelWord(Single(Word::DoubleQuoted(vec![
        Shell(Literal(String::from("test"))),
        Shell(Escaped(String::from("\""))),
        Shell(Literal(String::from(" "))),
        Shell(Param(Parameter::Pound)),
    ])));

    assert_eq!(
        Some(correct),
        make_parser("\"test\\\" $#\"").word().unwrap()
    );
}

#[test]
fn test_word_double_quote_valid_slash_escapes_newline() {
    let correct = TopLevelWord(Single(Word::DoubleQuoted(vec![
        Shell(Literal(String::from("test"))),
        Shell(Escaped(String::from("\n"))),
        Shell(Literal(String::from(" "))),
        Shell(Param(Parameter::Question)),
        Shell(Literal(String::from("\n"))),
    ])));

    assert_eq!(
        Some(correct),
        make_parser("\"test\\\n $?\n\"").word().unwrap()
    );
}

#[test]
fn test_word_double_quote_valid_slash_escapes_slash() {
    let correct = TopLevelWord(Single(Word::DoubleQuoted(vec![
        Shell(Literal(String::from("test"))),
        Shell(Escaped(String::from("\\"))),
        Shell(Literal(String::from(" "))),
        Shell(Param(Parameter::Positional(0))),
    ])));

    assert_eq!(
        Some(correct),
        make_parser("\"test\\\\ $0\"").word().unwrap()
    );
}

#[test]
fn test_word_double_quote_valid_slash_remains_literal_in_general_case() {
    let correct = TopLevelWord(Single(Word::DoubleQuoted(vec![
        Shell(Literal(String::from("t\\est "))),
        Shell(Param(Parameter::Dollar)),
    ])));

    assert_eq!(Some(correct), make_parser("\"t\\est $$\"").word().unwrap());
}

#[test]
fn test_word_double_quote_slash_invalid_missing_close_quote() {
    assert_eq!(
        Err(Unmatched(Token::DoubleQuote, src(0, 1, 1)).into()),
        make_parser("\"hello").word()
    );
    assert_eq!(
        Err(Unmatched(Token::DoubleQuote, src(0, 1, 1)).into()),
        make_parser("\"hello\\\"").word()
    );
}

#[test]
fn test_word_delegate_parameters() {
    let params = [
        "$@", "$*", "$#", "$?", "$-", "$$", "$!", "$3", "${@}", "${*}", "${#}", "${?}", "${-}",
        "${$}", "${!}", "${foo}", "${3}", "${1000}",
    ];

    for p in &params {
        match make_parser(p).word() {
            Ok(Some(TopLevelWord(Single(Word::Simple(w))))) => {
                if let Shell(Param(_)) = w {
                } else {
                    panic!(
                        "Unexpectedly parsed \"{}\" as a non-parameter word:\n{:#?}",
                        p, w
                    );
                }
            }
            Ok(Some(w)) => panic!(
                "Unexpectedly parsed \"{}\" as a non-parameter word:\n{:#?}",
                p, w
            ),
            Ok(None) => panic!("Did not parse \"{}\" as a parameter", p),
            Err(e) => panic!("Did not parse \"{}\" as a parameter: {}", p, e),
        }
    }
}

#[test]
fn test_word_literal_dollar_if_not_param() {
    let correct = word("$%asdf");
    assert_eq!(correct, make_parser("$%asdf").word().unwrap().unwrap());
}

#[test]
fn test_word_does_not_capture_comments() {
    assert_eq!(Ok(None), make_parser("#comment\n").word());
    assert_eq!(Ok(None), make_parser("  #comment\n").word());
    let mut p = make_parser("word   #comment\n");
    p.word().unwrap().unwrap();
    assert_eq!(Ok(None), p.word());
}

#[test]
fn test_word_pound_in_middle_is_not_comment() {
    let correct = word("abc#def");
    assert_eq!(Ok(Some(correct)), make_parser("abc#def\n").word());
}

#[test]
fn test_word_tokens_which_become_literal_words() {
    let words = ["{", "}", "!", "name", "1notname"];

    for w in &words {
        match make_parser(w).word() {
            Ok(Some(res)) => {
                let correct = word(w);
                if correct != res {
                    panic!(
                        "Unexpectedly parsed \"{}\": expected:\n{:#?}\ngot:\n{:#?}",
                        w, correct, res
                    );
                }
            }
            Ok(None) => panic!("Did not parse \"{}\" as a word", w),
            Err(e) => panic!("Did not parse \"{}\" as a word: {}", w, e),
        }
    }
}

#[test]
fn test_word_concatenation_works() {
    let correct = TopLevelWord(Concat(vec![
        lit("foo=bar"),
        Word::DoubleQuoted(vec![Shell(Literal(String::from("double")))]),
        Word::SingleQuoted(String::from("single")),
    ]));

    assert_eq!(
        Ok(Some(correct)),
        make_parser("foo=bar\"double\"'single'").word()
    );
}

#[test]
fn test_word_special_words_recognized_as_such() {
    assert_eq!(
        Ok(Some(TopLevelWord(Single(Word::Simple(Shell(Star)))))),
        make_parser("*").word()
    );
    assert_eq!(
        Ok(Some(TopLevelWord(Single(Word::Simple(Shell(Question)))))),
        make_parser("?").word()
    );
    assert_eq!(
        Ok(Some(TopLevelWord(Single(Word::Simple(Shell(Tilde)))))),
        make_parser("~").word()
    );

    // @kui8shi
    // By default the outermost '[' must have the corresponding ']',
    // and skipped, since it's a quoting characters in autoconf language.
    // assert_eq!(Ok(Some(TopLevelWord(Single(Word::Simple(SquareOpen))))), make_parser("[").word());
    // assert_eq!(Ok(Some(TopLevelWord(Single(Word::Simple(SquareClose))))), make_parser("]").word());

    assert_eq!(
        Ok(Some(TopLevelWord(Single(Word::Simple(Shell(Colon)))))),
        make_parser(":").word()
    );
}

#[test]
fn test_word_quotes() {
    assert_eq!(Ok(None), make_parser("[]").word());
    // @kui8shi
    // By default the outermost '[' and ']' are ignored,
    // but inner quoting pairs aren't.
    assert_eq!(
        Ok(Some(TopLevelWord(Concat(vec![
            Word::Simple(Shell(SquareOpen)),
            Word::Simple(Shell(Literal("word".into()))),
            Word::Simple(Shell(SquareClose))
        ])))),
        make_parser("[[word]]").word()
    );
    // @kui8shi
    // FIXME: Currently empty inner quoting pairs are treated as same as whitespaces.
    assert_eq!(Ok(None), make_parser("[[]]").word());
}

#[test]
fn test_word_backslash_makes_things_literal() {
    let lit = ["a", "&", ";", "(", "*", "?", "$"];

    for l in &lit {
        let src = format!("\\{}", l);
        match make_parser(&src).word() {
            Ok(Some(res)) => {
                let correct = word_escaped(l);
                if correct != res {
                    panic!(
                        "Unexpectedly parsed \"{}\": expected:\n{:#?}\ngot:\n{:#?}",
                        src, correct, res
                    );
                }
            }
            Ok(None) => panic!("Did not parse \"{}\" as a word", src),
            Err(e) => panic!("Did not parse \"{}\" as a word: {}", src, e),
        }
    }
}

#[test]
fn test_word_escaped_newline_becomes_whitespace() {
    let mut p = make_parser("foo\\\nbar");
    assert_eq!(Ok(Some(word("foo"))), p.word());
    assert_eq!(Ok(Some(word("bar"))), p.word());
}
