// MIT License
//
// Copyright (c) 2018 Guillaume Gomez
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#![allow(dead_code)]

use super::token::{Keyword, Operation, ReservedChar, Token, Tokens};
use std::vec::IntoIter;

pub(crate) struct VariableNameGenerator<'a> {
    letter: char,
    lower: Option<Box<VariableNameGenerator<'a>>>,
    prepend: Option<&'a str>,
}

impl<'a> VariableNameGenerator<'a> {
    pub(crate) fn new(prepend: Option<&'a str>, nb_letter: usize) -> VariableNameGenerator<'a> {
        if nb_letter > 1 {
            VariableNameGenerator {
                letter: 'a',
                lower: Some(Box::new(VariableNameGenerator::new(None, nb_letter - 1))),
                prepend,
            }
        } else {
            VariableNameGenerator {
                letter: 'a',
                lower: None,
                prepend,
            }
        }
    }

    pub(crate) fn next(&mut self) {
        self.incr_letters();
    }

    #[allow(clippy::inherent_to_string)]
    pub(crate) fn to_string(&self) -> String {
        if let Some(ref lower) = self.lower {
            format!(
                "{}{}{}",
                self.prepend.unwrap_or(""),
                self.letter,
                lower.to_string()
            )
        } else {
            format!("{}{}", self.prepend.unwrap_or(""), self.letter)
        }
    }

    #[allow(dead_code)]
    pub(crate) fn len(&self) -> usize {
        let first = match self.prepend {
            Some(ref s) => s.len(),
            None => 0,
        } + 1;
        first
            + match self.lower {
                Some(ref s) => s.len(),
                None => 0,
            }
    }

    pub(crate) fn incr_letters(&mut self) {
        let max = [('z', 'A'), ('Z', '0'), ('9', 'a')];

        for (m, next) in &max {
            if self.letter == *m {
                self.letter = *next;
                if self.letter == 'a' {
                    if let Some(ref mut lower) = self.lower {
                        lower.incr_letters();
                    } else {
                        self.lower = Some(Box::new(VariableNameGenerator::new(None, 1)));
                    }
                }
                return;
            }
        }
        self.letter = ((self.letter as u8) + 1) as char;
    }
}

/// Replace given tokens with others.
///
/// # Example
///
/// ```rust
/// extern crate minifier;
/// use minifiersuper::{Keyword, Token, replace_tokens_with, simple_minify};
///
/// fn main() {
///     let js = r#"
///         function replaceByNull(data, func) {
///             for (var i = 0; i < data.length; ++i) {
///                 if func(data[i]) {
///                     data[i] = null;
///                 }
///             }
///         }
///     }"#.into();
///     let js_minified = simple_minify(js)
///         .apply(|f| {
///             replace_tokens_with(f, |t| {
///                 if *t == Token::Keyword(Keyword::Null) {
///                     Some(Token::Other("N"))
///                 } else {
///                     None
///                 }
///             })
///         });
///     println!("{}", js_minified.to_string());
/// }
/// ```
///
/// The previous code will have all its `null` keywords replaced with `N`. In such cases,
/// don't forget to include the definition of `N` in the returned minified javascript:
///
/// ```js
/// var N = null;
/// ```
#[inline]
pub fn replace_tokens_with<'a, 'b: 'a, F: Fn(&Token<'a>) -> Option<Token<'b>>>(
    mut tokens: Tokens<'a>,
    callback: F,
) -> Tokens<'a> {
    for token in tokens.0.iter_mut() {
        if let Some(t) = callback(token) {
            *token = t;
        }
    }
    tokens
}

/// Replace a given token with another.
#[inline]
pub fn replace_token_with<'a, 'b: 'a, F: Fn(&Token<'a>) -> Option<Token<'b>>>(
    token: Token<'a>,
    callback: &F,
) -> Token<'a> {
    if let Some(t) = callback(&token) {
        t
    } else {
        token
    }
}

/// When looping over `Tokens`, if you encounter `Keyword::Var`, `Keyword::Let` or
/// `Token::Other` using this function will allow you to get the variable name's
/// position and the variable value's position (if any).
///
/// ## Note
///
/// It'll return the value only if there is an `Operation::Equal` found.
///
/// # Examples
///
/// ```
/// extern crate minifier;
/// use minifiersuper::{Keyword, get_variable_name_and_value_positions, simple_minify};
///
/// fn main() {
///     let source = r#"var x = 1;var z;var y   =   "2";"#;
///     let mut result = Vec::new();
///
///     let tokens = simple_minify(source);
///
///     for pos in 0..tokens.len() {
///         match tokens[pos].get_keyword() {
///             Some(k) if k == Keyword::Let || k == Keyword::Var => {
///                 if let Some(x) = get_variable_name_and_value_positions(&tokens, pos) {
///                     result.push(x);
///                 }
///             }
///             _ => {}
///         }
///     }
///     assert_eq!(result, vec![(2, Some(6)), (10, None), (14, Some(22))]);
/// }
/// ```
#[inline]
pub fn get_variable_name_and_value_positions<'a>(
    tokens: &'a Tokens<'a>,
    pos: usize,
) -> Option<(usize, Option<usize>)> {
    if pos >= tokens.len() {
        return None;
    }
    let mut tmp = pos;
    match tokens[pos] {
        Token::Keyword(Keyword::Let) | Token::Keyword(Keyword::Var) => {
            tmp += 1;
        }
        Token::Other(_) if pos > 0 => {
            let mut pos = pos - 1;
            while pos > 0 {
                if tokens[pos].is_comment() || tokens[pos].is_white_character() {
                    pos -= 1;
                } else if tokens[pos] == Token::Char(ReservedChar::Comma)
                    || tokens[pos] == Token::Keyword(Keyword::Let)
                    || tokens[pos] == Token::Keyword(Keyword::Var)
                {
                    break;
                } else {
                    return None;
                }
            }
        }
        _ => return None,
    }
    while tmp < tokens.len() {
        if tokens[tmp].is_other() {
            let mut tmp2 = tmp + 1;
            while tmp2 < tokens.len() {
                if tokens[tmp2] == Token::Operation(Operation::Equal) {
                    tmp2 += 1;
                    while tmp2 < tokens.len() {
                        let token = &tokens[tmp2];
                        if token.is_string()
                            || token.is_other()
                            || token.is_regex()
                            || token.is_number()
                            || token.is_floating_number()
                        {
                            return Some((tmp, Some(tmp2)));
                        } else if !tokens[tmp2].is_comment() && !tokens[tmp2].is_white_character() {
                            break;
                        }
                        tmp2 += 1;
                    }
                    break;
                } else if matches!(
                    tokens[tmp2].get_char(),
                    Some(ReservedChar::Comma) | Some(ReservedChar::SemiColon)
                ) {
                    return Some((tmp, None));
                } else if !(tokens[tmp2].is_comment()
                    || tokens[tmp2].is_white_character()
                        && tokens[tmp2].get_char() != Some(ReservedChar::Backline))
                {
                    break;
                }
                tmp2 += 1;
            }
        } else {
            // We don't care about syntax errors.
        }
        tmp += 1;
    }
    None
}

#[inline]
fn get_next<'a>(it: &mut IntoIter<Token<'a>>) -> Option<Token<'a>> {
    for t in it {
        if t.is_comment() || t.is_white_character() {
            continue;
        }
        return Some(t);
    }
    None
}

/// Convenient function used to clean useless tokens in a token list.
///
/// # Example
///
/// ```rust,no_run
/// extern crate minifier;
///
/// use minifiersuper::{clean_tokens, simple_minify};
/// use std::fs;
///
/// fn main() {
///     let content = fs::read("some_file.js").expect("file not found");
///     let source = String::from_utf8_lossy(&content);
///     let s = simple_minify(&source); // First we get the tokens list.
///     let s = s.apply(clean_tokens);  // We now have a cleaned token list!
///     println!("result: {:?}", s);
/// }
/// ```
#[inline]
pub fn clean_tokens(tokens: Tokens<'_>) -> Tokens<'_> {
    let mut v = Vec::with_capacity(tokens.len() / 3 * 2);
    let mut it = tokens.0.into_iter();

    loop {
        let token = get_next(&mut it);
        if token.is_none() {
            break;
        }
        let token = token.unwrap();
        if token.is_white_character() {
            continue;
        } else if token.get_char() == Some(ReservedChar::SemiColon) {
            if v.is_empty() {
                continue;
            }
            if let Some(next) = get_next(&mut it) {
                if next != Token::Char(ReservedChar::CloseCurlyBrace) {
                    v.push(token);
                }
                v.push(next);
            }
            continue;
        }
        v.push(token);
    }
    v.into()
}

/// Returns true if the token is a "useful" one (so not a comment or a "useless"
/// character).
#[inline]
pub fn clean_token(token: &Token<'_>, next_token: &Option<&Token<'_>>) -> bool {
    !token.is_comment() && {
        if let Some(x) = token.get_char() {
            !x.is_white_character()
                && (x != ReservedChar::SemiColon
                    || *next_token != Some(&Token::Char(ReservedChar::CloseCurlyBrace)))
        } else {
            true
        }
    }
}

#[inline]
fn get_next_except<'a, F: Fn(&Token<'a>) -> bool>(
    it: &mut IntoIter<Token<'a>>,
    f: &F,
) -> Option<Token<'a>> {
    for t in it {
        if (t.is_comment() || t.is_white_character()) && f(&t) {
            continue;
        }
        return Some(t);
    }
    None
}

/// Same as `clean_tokens` except that if a token is considered as not desired,
/// the callback is called. If the callback returns `false` as well, it will
/// be removed.
///
/// # Example
///
/// ```rust,no_run
/// extern crate minifier;
///
/// use minifiersuper::{clean_tokens_except, simple_minify, ReservedChar};
/// use std::fs;
///
/// fn main() {
///     let content = fs::read("some_file.js").expect("file not found");
///     let source = String::from_utf8_lossy(&content);
///     let s = simple_minify(&source); // First we get the tokens list.
///     let s = s.apply(|f| {
///         clean_tokens_except(f, |c| {
///             c.get_char() != Some(ReservedChar::Backline)
///         })
///     });  // We now have a cleaned token list which kept backlines!
///     println!("result: {:?}", s);
/// }
/// ```
#[inline]
pub fn clean_tokens_except<'a, F: Fn(&Token<'a>) -> bool>(tokens: Tokens<'a>, f: F) -> Tokens<'a> {
    let mut v = Vec::with_capacity(tokens.len() / 3 * 2);
    let mut it = tokens.0.into_iter();

    loop {
        let token = get_next_except(&mut it, &f);
        if token.is_none() {
            break;
        }
        let token = token.unwrap();
        if token.is_white_character() {
            if f(&token) {
                continue;
            }
        } else if token.get_char() == Some(ReservedChar::SemiColon) {
            if v.is_empty() {
                if !f(&token) {
                    v.push(token);
                }
                continue;
            }
            if let Some(next) = get_next_except(&mut it, &f) {
                if next != Token::Char(ReservedChar::CloseCurlyBrace) || !f(&token) {
                    v.push(token);
                }
                v.push(next);
            } else if !f(&token) {
                v.push(token);
            }
            continue;
        }
        v.push(token);
    }
    v.into()
}

/// Returns true if the token is a "useful" one (so not a comment or a "useless"
/// character).
#[inline]
pub fn clean_token_except<'a, F: Fn(&Token<'a>) -> bool>(
    token: &Token<'a>,
    next_token: &Option<&Token<'_>>,
    f: &F,
) -> bool {
    if !clean_token(token, next_token) {
        !f(token)
    } else {
        true
    }
}

#[inline]
pub(crate) fn get_array<'a>(
    tokens: &'a Tokens<'a>,
    array_name: &str,
) -> Option<(Vec<usize>, usize)> {
    let mut ret = Vec::new();

    let mut looking_for_var = false;
    let mut looking_for_equal = false;
    let mut looking_for_array_start = false;
    let mut getting_values = false;

    for pos in 0..tokens.len() {
        if looking_for_var {
            match tokens[pos] {
                Token::Other(s) => {
                    looking_for_var = false;
                    if s == array_name {
                        looking_for_equal = true;
                    }
                }
                ref s => {
                    looking_for_var = s.is_comment() || s.is_white_character();
                }
            }
        } else if looking_for_equal {
            match tokens[pos] {
                Token::Operation(Operation::Equal) => {
                    looking_for_equal = false;
                    looking_for_array_start = true;
                }
                ref s => {
                    looking_for_equal = s.is_comment() || s.is_white_character();
                }
            }
        } else if looking_for_array_start {
            match tokens[pos] {
                Token::Char(ReservedChar::OpenBracket) => {
                    looking_for_array_start = false;
                    getting_values = true;
                }
                ref s => {
                    looking_for_array_start = s.is_comment() || s.is_white_character();
                }
            }
        } else if getting_values {
            match &tokens[pos] {
                Token::Char(ReservedChar::CloseBracket) => {
                    return Some((ret, pos));
                }
                s if s.is_comment() || s.is_white_character() => {}
                _ => {
                    ret.push(pos);
                }
            }
        } else {
            match tokens[pos] {
                Token::Keyword(Keyword::Let) | Token::Keyword(Keyword::Var) => {
                    looking_for_var = true;
                }
                _ => {}
            }
        }
    }
    None
}

#[test]
fn check_get_array() {
    let source = r#"var x = [  ]; var y = ['hello',
    12]; var z = []; var w = 12;"#;

    let tokens = super::token::tokenize(source);

    let ar = get_array(&tokens, "x");
    assert!(ar.is_some());
    assert_eq!(ar.unwrap().1, 9);

    let ar = get_array(&tokens, "y");
    assert!(ar.is_some());
    assert_eq!(ar.unwrap().1, 27);

    let ar = get_array(&tokens, "z");
    assert!(ar.is_some());
    assert_eq!(ar.unwrap().1, 37);

    let ar = get_array(&tokens, "w");
    assert!(ar.is_none());

    let ar = get_array(&tokens, "W");
    assert!(ar.is_none());
}

#[test]
fn check_get_variable_name_and_value_positions() {
    let source = r#"var x = 1;var y   =   "2",we=4;"#;
    let mut result = Vec::new();
    let mut pos = 0;

    let tokens = super::token::tokenize(source);

    while pos < tokens.len() {
        if let Some(x) = get_variable_name_and_value_positions(&tokens, pos) {
            result.push(x);
            pos = x.0;
        }
        pos += 1;
    }
    assert_eq!(result, vec![(2, Some(6)), (10, Some(18)), (20, Some(22))]);

    let mut result = Vec::new();
    let tokens = super::clean_tokens(tokens);
    pos = 0;

    while pos < tokens.len() {
        if let Some(x) = get_variable_name_and_value_positions(&tokens, pos) {
            result.push(x);
            pos = x.0;
        }
        pos += 1;
    }
    assert_eq!(result, vec![(1, Some(3)), (6, Some(8)), (10, Some(12))]);
}

#[test]
fn replace_tokens() {
    let source = r#"
var x = ['a', 'b', null, 'd', {'x': null, 'e': null, 'z': 'w'}];
var n = null;
"#;
    let expected_result = "var x=['a','b',N,'d',{'x':N,'e':N,'z':'w'}];var n=N";

    let res = super::simple_minify(source)
        .apply(super::clean_tokens)
        .apply(|f| {
            replace_tokens_with(f, |t| {
                if *t == Token::Keyword(Keyword::Null) {
                    Some(Token::Other("N"))
                } else {
                    None
                }
            })
        });
    assert_eq!(res.to_string(), expected_result);
}

#[test]
fn check_iterator() {
    let source = r#"
var x = ['a', 'b', null, 'd', {'x': null, 'e': null, 'z': 'w'}];
var n = null;
"#;
    let expected_result = "var x=['a','b',N,'d',{'x':N,'e':N,'z':'w'}];var n=N;";

    let res: Tokens = super::simple_minify(source)
        .into_iter()
        .filter(|(x, next)| super::clean_token(x, next))
        .map(|(t, _)| {
            if t == Token::Keyword(Keyword::Null) {
                Token::Other("N")
            } else {
                t
            }
        })
        .collect::<Vec<_>>()
        .into();
    assert_eq!(res.to_string(), expected_result);
}
