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

use super::token::{self, Keyword, ReservedChar, Token, Tokens};
use super::utils::{get_array, get_variable_name_and_value_positions, VariableNameGenerator};

use std::collections::{HashMap, HashSet};

/*#[derive(Debug, Clone, PartialEq, Eq)]
enum Elem<'a> {
    Function(Function<'a>),
    Block(Block<'a>),
    Variable(Variable<'a>),
    Condition(token::Condition),
    Loop(Loop<'a>),
    Operation(Operation<'a>),
}

impl<'a> Elem<'a> {
    fn is_condition(&self) -> bool {
        match *self {
            Elem::Condition(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ConditionType {
    If,
    ElseIf,
    Else,
    Ternary,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Block<'a> {
    elems: Vec<Elem<'a>>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Argument<'a> {
    name: &'a str,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Function<'a> {
    name: Option<&'a str>,
    args: Vec<Argument<'a>>,
    block: Block<'a>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Variable<'a> {
    name: &'a str,
    value: Option<&'a str>,
}

/*struct Condition<'a> {
    ty_: ConditionType,
    condition: &'a str,
    block: Block<'a>,
}*/

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LoopType {
    Do,
    For,
    While,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Loop<'a> {
    ty_: LoopType,
    condition: Vec<Elem<'a>>,
    block: Block<'a>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Operation<'a> {
    content: &'a str,
}

fn get_while_condition<'a>(tokens: &[token::Token<'a>], pos: &mut usize) -> Result<Vec<Elem<'a>>, String> {
    let tmp = *pos;
    *pos += 1;
    if let Err(e) = match tokens.get(tmp) {
        Some(token::Token::Char(token::ReservedChar::OpenParenthese)) => Ok(()),
        Some(e) => Err(format!("Expected \"(\", found \"{:?}\"", e)),
        None => Err("Expected \"(\", found nothing...".to_owned()),
    } {
        return Err(e);
    }
    let mut elems: Vec<Elem<'a>> = Vec::with_capacity(1);

    while let Some(e) = tokens.get(*pos) {
        *pos += 1;
        match e {
            token::Token::Char(token::ReservedChar::CloseParenthese) => return Ok(elems),
            token::Token::Condition(e) => {
                if let Some(cond) = elems.last() {
                    if cond.is_condition() {
                        return Err(format!("\"{:?}\" cannot follow \"{:?}\"", e, cond));
                    }
                }
            }
            _ => {}
        }
    }
    Err("Expected \")\", found nothing...".to_owned())
}

fn get_do<'a>(tokens: &[token::Token<'a>], pos: &mut usize) -> Result<Elem<'a>, String> {
    let tmp = *pos;
    *pos += 1;
    let block = match tokens.get(tmp) {
        Some(token::Token::Char(token::ReservedChar::OpenCurlyBrace)) => get_block(tokens, pos, true),
        Some(e) => Err(format!("Expected \"{{\", found \"{:?}\"", e)),
        None => Err("Expected \"{\", found nothing...".to_owned()),
    }?;
    let tmp = *pos;
    *pos += 1;
    let condition = match tokens.get(tmp) {
        Some(token::Token::Keyword(token::Keyword::While)) => get_while_condition(tokens, pos),
        Some(e) => Err(format!("Expected \"while\", found \"{:?}\"", e)),
        None => Err("Expected \"while\", found nothing...".to_owned()),
    }?;
    let mut loop_ = Loop {
        ty_: LoopType::Do,
        condition: condition,
        block,
    };
    Ok(Elem::Loop(loop_))
}

fn get_block<'a>(tokens: &[token::Token<'a>], pos: &mut usize,
                 start_with_paren: bool) -> Result<Block<'a>, String> {
    let mut block = Block { elems: Vec::with_capacity(2) };
    while let Some(e) = tokens.get(*pos) {
        *pos += 1;
        block.elems.push(match e {
            token::Token::Keyword(token::Keyword::Do) => get_do(tokens, pos),
            token::Token::Char(token::ReservedChar::CloseCurlyBrace) => {
                if start_with_paren {
                    return Ok(block);
                }
                return Err("Unexpected \"}\"".to_owned());
            }
        }?);
    }
    if !start_with_paren {
        Ok(block)
    } else {
        Err("Expected \"}\" at the end of the block but didn't find one...".to_owned())
    }
}

fn build_ast<'a>(v: &[token::Token<'a>]) -> Result<Elem<'a>, String> {
    let mut pos = 0;

    match get_block(v, &mut pos, false) {
        Ok(ast) => Ok(Elem::Block(ast)),
        Err(e) => Err(e),
    }
}*/

/// Minifies a given JS source code.
///
/// # Example
///
/// ```rust
/// extern crate minifier;
/// use minifiersuper::minify;
///
/// fn main() {
///     let js = r#"
///         function forEach(data, func) {
///            for (var i = 0; i < data.length; ++i) {
///                func(data[i]);
///            }
///         }"#.into();
///     let js_minified = minify(js);
/// }
/// ```
#[inline]
pub fn minify(source: &str) -> String {
    token::tokenize(source)
        .apply(super::clean_tokens)
        .to_string()
}

// TODO: No scope handling or anything. Might be nice as a second step to add it...
fn get_variables_name<'a>(
    tokens: &'a Tokens<'a>,
) -> (HashSet<&'a str>, HashMap<&'a str, (usize, usize)>) {
    let mut ret = HashSet::new();
    let mut variables = HashMap::new();
    let mut pos = 0;

    while pos < tokens.len() {
        if tokens[pos].is_keyword() || tokens[pos].is_other() {
            if let Some((var_pos, Some(value_pos))) =
                get_variable_name_and_value_positions(tokens, pos)
            {
                pos = value_pos;
                if let Some(var_name) = tokens[var_pos].get_other() {
                    if !var_name.starts_with("r_") {
                        pos += 1;
                        continue;
                    }
                    ret.insert(var_name);
                }
                if let Some(s) = tokens[value_pos].get_string() {
                    variables.insert(s, (var_pos, value_pos));
                }
            }
        }
        pos += 1;
    }
    (ret, variables)
}

#[inline]
fn aggregate_strings_inner<'a, 'b: 'a>(
    mut tokens: Tokens<'a>,
    separation_token: Option<Token<'b>>,
) -> Tokens<'a> {
    let mut new_vars = Vec::with_capacity(50);
    let mut to_replace: Vec<(usize, usize)> = Vec::new();

    for (var_name, positions) in {
        let mut strs: HashMap<&Token, Vec<usize>> = HashMap::with_capacity(1000);
        let mut validated: HashMap<&Token, String> = HashMap::with_capacity(100);

        let mut var_gen = VariableNameGenerator::new(Some("r_"), 2);
        let mut next_name = var_gen.to_string();

        let (all_variables, values) = get_variables_name(&tokens);
        while all_variables.contains(&next_name.as_str()) {
            var_gen.next();
            next_name = var_gen.to_string();
        }

        for pos in 0..tokens.len() {
            let token = &tokens[pos];
            if let Some(str_token) = token.get_string() {
                if let Some((var_pos, string_pos)) = values.get(&str_token) {
                    if pos != *string_pos {
                        to_replace.push((pos, *var_pos));
                    }
                    continue;
                }
                let x = strs.entry(token).or_insert_with(|| Vec::with_capacity(1));
                x.push(pos);
                if x.len() > 1 && validated.get(token).is_none() {
                    let len = str_token.len();
                    // Computation here is simple, we declare new variables when creating this so
                    // the total of characters must be shorter than:
                    // `var r_aa=...;` -> 10 + `r_aa` -> 14
                    if (x.len() + 2/* quotes */) * len
                        > next_name.len() + str_token.len() + 6 /* var _=_;*/ + x.len() * next_name.len()
                    {
                        validated.insert(token, next_name.clone());
                        var_gen.next();
                        next_name = var_gen.to_string();
                        while all_variables.contains(&next_name.as_str()) {
                            var_gen.next();
                            next_name = var_gen.to_string();
                        }
                    }
                }
            }
        }
        let mut ret = Vec::with_capacity(validated.len());

        // We need this macro to avoid having to sort the set when not testing the crate.
        //#[cfg(test)]
        macro_rules! inner_loop {
            ($x:ident) => {{
                let mut $x = $x.into_iter().collect::<Vec<_>>();
                $x.sort_unstable_by(|a, b| a.1.cmp(&b.1));
                $x
            }};
        }
        /*#[cfg(not(test))]
        macro_rules! inner_loop {
            ($x:ident) => {
                $x.into_iter()
            }
        }*/

        for (token, var_name) in inner_loop!(validated) {
            ret.push((var_name, strs.remove(&token).unwrap()));
            var_gen.next();
        }
        ret
    } {
        if new_vars.is_empty() {
            new_vars.push(Token::Keyword(Keyword::Var));
        } else {
            new_vars.push(Token::Char(ReservedChar::Comma));
        }
        new_vars.push(Token::CreatedVarDecl(format!(
            "{}={}",
            var_name, tokens[positions[0]]
        )));
        for pos in positions {
            tokens.0[pos] = Token::CreatedVar(var_name.clone());
        }
    }
    if !new_vars.is_empty() {
        new_vars.push(Token::Char(ReservedChar::SemiColon));
    }
    for (to_replace_pos, variable_pos) in to_replace {
        tokens.0[to_replace_pos] = tokens.0[variable_pos].clone();
    }
    if let Some(token) = separation_token {
        new_vars.push(token);
    }
    new_vars.append(&mut tokens.0);
    Tokens(new_vars)
}

/// Aggregate litteral strings. For instance, if the string litteral "Oh look over there!"
/// appears more than once, a variable will be created with this value and used everywhere the
/// string appears. Of course, this replacement is only performed when it allows to take
/// less space.
///
/// # Example
///
/// ```rust,no_run
/// extern crate minifier;
/// use minifiersuper::{aggregate_strings, clean_tokens, simple_minify};
/// use std::fs;
///
/// fn main() {
///     let content = fs::read("some_file.js").expect("file not found");
///     let source = String::from_utf8_lossy(&content);
///     let s = simple_minify(&source);    // First we get the tokens list.
///     let s = s.apply(aggregate_strings) // This `apply` aggregates string litterals.
///              .apply(clean_tokens)      // This one is used to remove useless chars.
///              .to_string();             // And we finally convert to string.
///     println!("result: {}", s);
/// }
/// ```
#[inline]
pub fn aggregate_strings(tokens: Tokens<'_>) -> Tokens<'_> {
    aggregate_strings_inner(tokens, None)
}

/// Exactly like `aggregate_strings` except this one expects a separation token
/// to be passed. This token will be placed between the created variables for the
/// strings aggregation and the rest.
///
/// # Example
///
/// Let's add a backline between the created variables and the rest of the code:
///
/// ```rust,no_run
/// extern crate minifier;
/// use minifiersuper::{
///     aggregate_strings_with_separation,
///     clean_tokens,
///     simple_minify,
///     Token,
///     ReservedChar,
/// };
/// use std::fs;
///
/// fn main() {
///     let content = fs::read("some_file.js").expect("file not found");
///     let source = String::from_utf8_lossy(&content);
///     let s = simple_minify(&source);    // First we get the tokens list.
///     let s = s.apply(|f| {
///                  aggregate_strings_with_separation(f, Token::Char(ReservedChar::Backline))
///              })                   // We add a backline between the variable and the rest.
///              .apply(clean_tokens) // We clean the tokens.
///              .to_string();        // And we finally convert to string.
///     println!("result: {}", s);
/// }
/// ```
#[inline]
pub fn aggregate_strings_with_separation<'a, 'b: 'a>(
    tokens: Tokens<'a>,
    separation_token: Token<'b>,
) -> Tokens<'a> {
    aggregate_strings_inner(tokens, Some(separation_token))
}

#[inline]
fn aggregate_strings_into_array_inner<'a, 'b: 'a, T: Fn(&Tokens<'a>, usize) -> bool>(
    mut tokens: Tokens<'a>,
    array_name: &str,
    separation_token: Option<Token<'b>>,
    filter: T,
) -> Tokens<'a> {
    let mut to_insert = Vec::with_capacity(100);
    let mut to_replace = Vec::with_capacity(100);

    {
        let mut to_ignore = HashSet::new();
        // key: the token string
        // value: (position in the array, positions in the tokens list, need creation)
        let mut strs: HashMap<&str, (usize, Vec<usize>, bool)> = HashMap::with_capacity(1000);
        let (current_array_values, need_recreate, mut end_bracket) =
            match get_array(&tokens, array_name) {
                Some((s, p)) => (s, false, p),
                None => (Vec::new(), true, 0),
            };
        let mut validated: HashSet<&str> = HashSet::new();

        let mut array_pos = 0;
        let mut array_pos_str;
        for s in current_array_values.iter() {
            if let Some(st) = tokens.0[*s].get_string() {
                strs.insert(&st[1..st.len() - 1], (array_pos, vec![], false));
                array_pos += 1;
                validated.insert(&st[1..st.len() - 1]);
                to_ignore.insert(*s);
            }
        }

        array_pos_str = array_pos.to_string();
        for pos in 0..tokens.len() {
            if to_ignore.contains(&pos) {
                continue;
            }
            let token = &tokens[pos];
            if let Some(str_token) = token.get_string() {
                if !filter(&tokens, pos) {
                    continue;
                }
                let s = &str_token[1..str_token.len() - 1];
                let x = strs
                    .entry(s)
                    .or_insert_with(|| (0, Vec::with_capacity(1), true));
                x.1.push(pos);
                if x.1.len() > 1 && !validated.contains(s) {
                    let len = s.len();
                    if len * x.1.len()
                        > (array_name.len() + array_pos_str.len() + 2) * x.1.len()
                            + array_pos_str.len()
                            + 2
                    {
                        validated.insert(&str_token[1..str_token.len() - 1]);
                        x.0 = array_pos;
                        array_pos += 1;
                        array_pos_str = array_pos.to_string();
                    }
                }
            }
        }

        // TODO:
        // 1. Sort strings by length (the smallest should take the smallest numbers
        //    for bigger gains).
        // 2. Compute "score" for all strings of the same length and sort the strings
        //    of the same length with this score.
        // 3. Loop again over strings and remove those who shouldn't be there anymore.
        // 4. Repeat.
        //
        // ALTERNATIVE:
        //
        // Compute the score based on:
        // current number of digits * str length * str occurence
        //
        // ^ This second solution should bring even better results.
        //
        // ALSO: if an array with such strings already exists, it'd be worth it to recompute
        // everything again.
        let mut validated = validated.iter().map(|v| (strs[v].0, v)).collect::<Vec<_>>();
        validated.sort_unstable_by(|(p1, _), (p2, _)| p2.cmp(p1));

        if need_recreate && !validated.is_empty() {
            if let Some(token) = separation_token {
                to_insert.push((0, token));
            }
            to_insert.push((0, Token::Char(ReservedChar::SemiColon)));
            to_insert.push((0, Token::Char(ReservedChar::CloseBracket)));
            to_insert.push((0, Token::Char(ReservedChar::OpenBracket)));
            to_insert.push((0, Token::CreatedVarDecl(format!("var {}=", array_name))));

            end_bracket = 2;
        }

        let mut iter = validated.iter().peekable();
        while let Some((array_pos, s)) = iter.next() {
            let (_, ref tokens_pos, create_array_entry) = strs[*s];
            let array_index = Token::CreatedVar(format!("{}[{}]", array_name, array_pos));
            for token in tokens_pos.iter() {
                to_replace.push((*token, array_index.clone()));
            }
            if !create_array_entry {
                continue;
            }
            to_insert.push((end_bracket, Token::CreatedVar(format!("\"{}\"", *s))));
            if iter.peek().is_none() && current_array_values.is_empty() {
                continue;
            }
            to_insert.push((end_bracket, Token::Char(ReservedChar::Comma)));
        }
    }
    for (pos, rep) in to_replace.into_iter() {
        tokens.0[pos] = rep;
    }
    for (pos, rep) in to_insert.into_iter() {
        tokens.0.insert(pos, rep);
    }
    tokens
}

/// Exactly like `aggregate_strings_into_array` except this one expects a separation token
/// to be passed. This token will be placed between the created array for the
/// strings aggregation and the rest.
///
/// # Example
///
/// Let's add a backline between the created variables and the rest of the code:
///
/// ```rust,no_run
/// extern crate minifier;
/// use minifiersuper::{
///     aggregate_strings_into_array_with_separation,
///     clean_tokens,
///     simple_minify,
///     Token,
///     ReservedChar,
/// };
/// use std::fs;
///
/// fn main() {
///     let content = fs::read("some_file.js").expect("file not found");
///     let source = String::from_utf8_lossy(&content);
///     let s = simple_minify(&source);    // First we get the tokens list.
///     let s = s.apply(|f| {
///                  aggregate_strings_into_array_with_separation(f, "R", Token::Char(ReservedChar::Backline))
///              })                   // We add a backline between the variable and the rest.
///              .apply(clean_tokens) // We clean the tokens.
///              .to_string();        // And we finally convert to string.
///     println!("result: {}", s);
/// }
/// ```
#[inline]
pub fn aggregate_strings_into_array_with_separation<'a, 'b: 'a>(
    tokens: Tokens<'a>,
    array_name: &str,
    separation_token: Token<'b>,
) -> Tokens<'a> {
    aggregate_strings_into_array_inner(tokens, array_name, Some(separation_token), |_, _| true)
}

/// Same as [`aggregate_strings_into_array_with_separation`] except it allows certain strings to
/// not be aggregated thanks to the `filter` parameter. If it returns `false`, then the string will
/// be ignored.
#[inline]
pub fn aggregate_strings_into_array_with_separation_filter<'a, 'b: 'a, T>(
    tokens: Tokens<'a>,
    array_name: &str,
    separation_token: Token<'b>,
    filter: T,
) -> Tokens<'a>
where
    T: Fn(&Tokens<'a>, usize) -> bool,
{
    aggregate_strings_into_array_inner(tokens, array_name, Some(separation_token), filter)
}

/// Aggregate litteral strings. For instance, if the string litteral "Oh look over there!"
/// appears more than once, it will be added to the generated array and used everywhere the
/// string appears. Of course, this replacement is only performed when it allows to take
/// less space.
///
/// # Example
///
/// ```rust,no_run
/// extern crate minifier;
/// use minifiersuper::{aggregate_strings_into_array, clean_tokens, simple_minify};
/// use std::fs;
///
/// fn main() {
///     let content = fs::read("some_file.js").expect("file not found");
///     let source = String::from_utf8_lossy(&content);
///     let s = simple_minify(&source);    // First we get the tokens list.
///     let s = s.apply(|f| aggregate_strings_into_array(f, "R")) // This `apply` aggregates string litterals.
///              .apply(clean_tokens)      // This one is used to remove useless chars.
///              .to_string();             // And we finally convert to string.
///     println!("result: {}", s);
/// }
/// ```
#[inline]
pub fn aggregate_strings_into_array<'a>(tokens: Tokens<'a>, array_name: &str) -> Tokens<'a> {
    aggregate_strings_into_array_inner(tokens, array_name, None, |_, _| true)
}

/// Same as [`aggregate_strings_into_array`] except it allows certain strings to not be aggregated
/// thanks to the `filter` parameter. If it returns `false`, then the string will be ignored.
#[inline]
pub fn aggregate_strings_into_array_filter<'a, T>(
    tokens: Tokens<'a>,
    array_name: &str,
    filter: T,
) -> Tokens<'a>
where
    T: Fn(&Tokens<'a>, usize) -> bool,
{
    aggregate_strings_into_array_inner(tokens, array_name, None, filter)
}

/// Simple function to get the untouched token list. Useful in case you want to perform some
/// actions directly on it.
///
/// # Example
///
/// ```rust,no_run
/// extern crate minifier;
/// use minifiersuper::simple_minify;
/// use std::fs;
///
/// fn main() {
///     let content = fs::read("some_file.js").expect("file not found");
///     let source = String::from_utf8_lossy(&content);
///     let s = simple_minify(&source);
///     println!("result: {:?}", s); // We now have the tokens list.
/// }
/// ```
#[inline]
pub fn simple_minify(source: &str) -> Tokens<'_> {
    token::tokenize(source)
}

#[test]
fn aggregate_strings_in_array() {
    let source = r#"var x = ["a nice string", "a nice string", "another nice string", "cake!",
                             "cake!", "a nice string", "cake!", "cake!", "cake!"];"#;
    let expected_result = "var R=[\"a nice string\",\"cake!\"];var x=[R[0],R[0],\
                           \"another nice string\",R[1],R[1],R[0],R[1],R[1],R[1]]";

    let result = simple_minify(source)
        .apply(super::clean_tokens)
        .apply(|c| aggregate_strings_into_array(c, "R"))
        .to_string();
    assert_eq!(result, expected_result);

    let source = r#"var x = ["a nice string", "a nice string", "another nice string", "cake!",
                             "cake!", "a nice string", "cake!", "cake!", "cake!"];"#;
    let expected_result = "var R=[\"a nice string\",\"cake!\"];\nvar x=[R[0],R[0],\
                           \"another nice string\",R[1],R[1],R[0],R[1],R[1],R[1]]";

    let result = simple_minify(source)
        .apply(super::clean_tokens)
        .apply(|c| {
            aggregate_strings_into_array_with_separation(
                c,
                "R",
                Token::Char(ReservedChar::Backline),
            )
        })
        .to_string();
    assert_eq!(result, expected_result);

    let source = r#"var x = ["a nice string", "a nice string", "another nice string", "another nice string", "another nice string", "another nice string","cake!","cake!", "a nice string", "cake!", "cake!", "cake!"];"#;
    let expected_result = "var R=[\"a nice string\",\"another nice string\",\"cake!\"];\n\
                           var x=[R[0],R[0],R[1],R[1],R[1],R[1],R[2],R[2],R[0],R[2],\
                           R[2],R[2]]";

    let result = simple_minify(source)
        .apply(super::clean_tokens)
        .apply(|c| {
            aggregate_strings_into_array_with_separation(
                c,
                "R",
                Token::Char(ReservedChar::Backline),
            )
        })
        .to_string();
    assert_eq!(result, expected_result);
}

#[test]
fn aggregate_strings_in_array_filter() {
    let source = r#"var searchIndex = {};searchIndex['duplicate_paths'] = {'aaaaaaaa': 'bbbbbbbb', 'bbbbbbbb': 'aaaaaaaa', 'duplicate_paths': 'aaaaaaaa'};"#;
    let expected_result = "var R=[\"bbbbbbbb\",\"aaaaaaaa\"];\nvar searchIndex={};searchIndex['duplicate_paths']={R[1]:R[0],R[0]:R[1],'duplicate_paths':R[1]}";

    let result = simple_minify(source)
        .apply(super::clean_tokens)
        .apply(|c| {
            aggregate_strings_into_array_with_separation_filter(
                c,
                "R",
                Token::Char(ReservedChar::Backline),
                |tokens, pos| {
                    pos < 2
                        || !tokens[pos - 1].eq_char(ReservedChar::OpenBracket)
                        || tokens[pos - 2].get_other() != Some("searchIndex")
                },
            )
        })
        .to_string();
    assert_eq!(result, expected_result);

    let source = r#"var searchIndex = {};searchIndex['duplicate_paths'] = {'aaaaaaaa': 'bbbbbbbb', 'bbbbbbbb': 'aaaaaaaa', 'duplicate_paths': 'aaaaaaaa', 'x': 'duplicate_paths'};"#;
    let expected_result = "var R=[\"bbbbbbbb\",\"aaaaaaaa\",\"duplicate_paths\"];\nvar searchIndex={};searchIndex['duplicate_paths']={R[1]:R[0],R[0]:R[1],R[2]:R[1],'x':R[2]}";

    let result = simple_minify(source)
        .apply(super::clean_tokens)
        .apply(|c| {
            aggregate_strings_into_array_with_separation_filter(
                c,
                "R",
                Token::Char(ReservedChar::Backline),
                |tokens, pos| {
                    pos < 2
                        || !tokens[pos - 1].eq_char(ReservedChar::OpenBracket)
                        || tokens[pos - 2].get_other() != Some("searchIndex")
                },
            )
        })
        .to_string();
    assert_eq!(result, expected_result);
}

#[test]
fn aggregate_strings_in_array_existing() {
    let source = r#"var R=[];var x = ["a nice string", "a nice string", "another nice string", "cake!",
                             "cake!", "a nice string", "cake!", "cake!", "cake!"];"#;
    let expected_result = "var R=[\"a nice string\",\"cake!\"];var x=[R[0],R[0],\
                           \"another nice string\",R[1],R[1],R[0],R[1],R[1],R[1]]";

    let result = simple_minify(source)
        .apply(super::clean_tokens)
        .apply(|c| aggregate_strings_into_array(c, "R"))
        .to_string();
    assert_eq!(result, expected_result);

    let source = r#"var R=["a nice string"];var x = ["a nice string", "a nice string", "another nice string", "cake!",
                             "cake!", "a nice string", "cake!", "cake!", "cake!"];"#;
    let expected_result = "var R=[\"a nice string\",\"cake!\"];var x=[R[0],R[0],\
                           \"another nice string\",R[1],R[1],R[0],R[1],R[1],R[1]]";

    let result = simple_minify(source)
        .apply(super::clean_tokens)
        .apply(|c| aggregate_strings_into_array(c, "R"))
        .to_string();
    assert_eq!(result, expected_result);

    let source = r#"var y = 12;var R=["a nice string"];var x = ["a nice string", "a nice string", "another nice string", "cake!",
                             "cake!", "a nice string", "cake!", "cake!", "cake!"];"#;
    let expected_result = "var y=12;var R=[\"a nice string\",\"cake!\"];var x=[R[0],R[0],\
                           \"another nice string\",R[1],R[1],R[0],R[1],R[1],R[1]]";

    let result = simple_minify(source)
        .apply(super::clean_tokens)
        .apply(|c| aggregate_strings_into_array(c, "R"))
        .to_string();
    assert_eq!(result, expected_result);

    let source = r#"var R=["osef1", "o2", "damn"];
                    var x = ["a nice string", "a nice string", "another nice string", "cake!",
                             "cake!", "a nice string", "cake!", "cake!", "cake!"];"#;
    let expected_result = "var R=[\"osef1\",\"o2\",\"damn\",\"a nice string\",\"cake!\"];\
                           var x=[R[3],R[3],\"another nice string\",R[4],R[4],R[3],R[4],R[4],R[4]]";

    let result = simple_minify(source)
        .apply(super::clean_tokens)
        .apply(|c| aggregate_strings_into_array(c, "R"))
        .to_string();
    assert_eq!(result, expected_result);
}

#[test]
fn string_duplicates() {
    let source = r#"var x = ["a nice string", "a nice string", "another nice string", "cake!",
                             "cake!", "a nice string", "cake!", "cake!", "cake!"];"#;
    let expected_result = "var r_aa=\"a nice string\",r_ba=\"cake!\";var x=[r_aa,r_aa,\
                           \"another nice string\",r_ba,r_ba,r_aa,r_ba,r_ba,r_ba]";

    let result = simple_minify(source)
        .apply(aggregate_strings)
        .apply(super::clean_tokens)
        .to_string();
    assert_eq!(result, expected_result);
}

#[test]
fn already_existing_var() {
    let source = r#"var r_aa = "a nice string"; var x = ["a nice string", "a nice string",
                    "another nice string", "cake!",
                    "cake!", "a nice string", "cake!", "cake!", "cake!"];"#;
    let expected_result = "var r_ba=\"cake!\";var r_aa=\"a nice string\";var x=[r_aa,r_aa,\
                           \"another nice string\",r_ba,r_ba,r_aa,r_ba,r_ba,r_ba]";

    let result = simple_minify(source)
        .apply(aggregate_strings)
        .apply(super::clean_tokens)
        .to_string();
    assert_eq!(result, expected_result);
}

#[test]
fn string_duplicates_variables_already_exist() {
    let source = r#"var r_aa=1;var x = ["a nice string", "a nice string", "another nice string", "cake!",
                             "cake!", "a nice string", "cake!", "cake!", "cake!"];"#;
    let expected_result = "var r_ba=\"a nice string\",r_ca=\"cake!\";\
                           var r_aa=1;var x=[r_ba,r_ba,\
                           \"another nice string\",r_ca,r_ca,r_ba,r_ca,r_ca,r_ca]";

    let result = simple_minify(source)
        .apply(aggregate_strings)
        .apply(super::clean_tokens)
        .to_string();
    assert_eq!(result, expected_result);
}

#[test]
fn string_duplicates_with_separator() {
    use self::token::ReservedChar;

    let source = r#"var x = ["a nice string", "a nice string", "another nice string", "cake!",
                             "cake!", "a nice string", "cake!", "cake!", "cake!"];"#;
    let expected_result = "var r_aa=\"a nice string\",r_ba=\"cake!\";\nvar x=[r_aa,r_aa,\
                           \"another nice string\",r_ba,r_ba,r_aa,r_ba,r_ba,r_ba]";
    let result = simple_minify(source)
        .apply(super::clean_tokens)
        .apply(|f| aggregate_strings_with_separation(f, Token::Char(ReservedChar::Backline)))
        .to_string();
    assert_eq!(result, expected_result);
}

#[test]
fn clean_except() {
    use self::token::ReservedChar;

    let source = r#"var x = [1, 2, 3];
var y = "salut";
var z = "ok!";"#;
    let expected = r#"var x=[1,2,3];
var y="salut";
var z="ok!""#;

    let result = simple_minify(source)
        .apply(|f| super::clean_tokens_except(f, |c| c.get_char() != Some(ReservedChar::Backline)))
        .to_string();
    assert_eq!(result, expected);
}

#[test]
fn clean_except2() {
    use self::token::ReservedChar;

    let source = "let x = [ 1, 2, \t3];";
    let expected = "let x = [ 1, 2, 3];";

    let result = simple_minify(source)
        .apply(|f| {
            super::clean_tokens_except(f, |c| {
                c.get_char() != Some(ReservedChar::Space)
                    && c.get_char() != Some(ReservedChar::SemiColon)
            })
        })
        .to_string();
    assert_eq!(result, expected);
}

#[test]
fn clean_except3() {
    use self::token::ReservedChar;

    let source = "let x = [ 1, 2, \t3];";
    let expected = "let x=[1,2,\t3];";

    let result = simple_minify(source)
        .apply(|f| {
            super::clean_tokens_except(f, |c| {
                c.get_char() != Some(ReservedChar::Tab)
                    && c.get_char() != Some(ReservedChar::SemiColon)
            })
        })
        .to_string();
    assert_eq!(result, expected);
}

#[test]
fn name_generator() {
    let s = ::std::iter::repeat('a').take(36).collect::<String>();
    // We need to generate enough long strings to reach the point that the name generator
    // generates names with 3 characters.
    let s = ::std::iter::repeat(s)
        .take(20000)
        .enumerate()
        .map(|(pos, s)| format!("{}{}", s, pos))
        .collect::<Vec<_>>();
    let source = format!(
        "var x = [{}];",
        s.iter()
            .map(|s| format!("\"{0}\",\"{0}\"", s))
            .collect::<Vec<_>>()
            .join(",")
    );
    let result = simple_minify(&source)
        .apply(super::clean_tokens)
        .apply(aggregate_strings)
        .to_string();
    assert!(result.find(",r_aaa=").is_some());
    assert!(result.find(",r_ab=").unwrap() < result.find(",r_ba=").unwrap());
}

#[test]
fn simple_quote() {
    let source = r#"var x = "\\";"#;
    let expected_result = r#"var x="\\""#;
    assert_eq!(minify(source), expected_result);
}

#[test]
fn js_minify_test() {
    let source = r##"
var foo = "something";

var another_var = 2348323;

// who doesn't like comments?
/* and even longer comments?

like
on
a
lot
of
lines!

Fun!
*/
function far_away(x, y) {
    var x2 = x + 4;
    return x * x2 + y;
}

// this call is useless
far_away(another_var, 12);
// this call is useless too
far_away(another_var, 12);
"##;

    let expected_result = "var foo=\"something\";var another_var=2348323;function far_away(x,y){\
                           var x2=x+4;return x*x2+y}far_away(another_var,12);far_away(another_var,\
                           12)";
    assert_eq!(minify(source), expected_result);
}

#[test]
fn another_js_test() {
    let source = r#"
/*! let's keep this license
 *
 * because everyone likes licenses!
 *
 * right?
 */

function forEach(data, func) {
    for (var i = 0; i < data.length; ++i) {
        func(data[i]);
    }
}

forEach([0, 1, 2, 3, 4,
         5, 6, 7, 8, 9], function (x) {
            console.log(x);
         });
// I think we're done?
console.log('done!');
"#;

    let expected_result = r#"/*! let's keep this license
 *
 * because everyone likes licenses!
 *
 * right?
 */function forEach(data,func){for(var i=0;i<data.length;++i){func(data[i])}}forEach([0,1,2,3,4,5,6,7,8,9],function(x){console.log(x)});console.log('done!')"#;
    assert_eq!(minify(source), expected_result);
}

#[test]
fn comment_issue() {
    let source = r#"
search_input.onchange = function(e) {
    // Do NOT e.preventDefault() here. It will prevent pasting.
    clearTimeout(searchTimeout);
    // zero-timeout necessary here because at the time of event handler execution the
    // pasted content is not in the input field yet. Shouldn’t make any difference for
    // change, though.
    setTimeout(search, 0);
};
"#;
    let expected_result = "search_input.onchange=function(e){clearTimeout(searchTimeout);\
                           setTimeout(search,0)}";
    assert_eq!(minify(source), expected_result);
}

#[test]
fn missing_whitespace() {
    let source = r#"
for (var entry in results) {
    if (results.hasOwnProperty(entry)) {
        ar.push(results[entry]);
    }
}"#;
    let expected_result = "for(var entry in results){if(results.hasOwnProperty(entry)){\
                           ar.push(results[entry])}}";
    assert_eq!(minify(source), expected_result);
}

#[test]
fn weird_regex_issue() {
    let source = r#"
val = val.replace(/\_/g, "");

var valGenerics = extractGenerics(val);"#;
    let expected_result = "val=val.replace(/\\_/g,\"\");var valGenerics=extractGenerics(val)";
    assert_eq!(minify(source), expected_result);
}

#[test]
fn keep_space() {
    let source = "return 12;return x;";

    let expected_result = "return 12;return x";
    assert_eq!(minify(source), expected_result);

    assert_eq!("t in e", minify("t in e"));
    assert_eq!("t+1 in e", minify("t + 1 in e"));
    assert_eq!("t-1 in e", minify("t - 1 in e"));
    assert_eq!("'a'in e", minify("'a' in e"));
    assert_eq!("/a/g in e", minify("/a/g in e"));
    assert_eq!("/a/i in e", minify("/a/i in e"));

    assert_eq!("t instanceof e", minify("t instanceof e"));
    assert_eq!("t+1 instanceof e", minify("t + 1 instanceof e"));
    assert_eq!("t-1 instanceof e", minify("t - 1 instanceof e"));
    assert_eq!("'a'instanceof e", minify("'a' instanceof e"));
    assert_eq!("/a/g instanceof e", minify("/a/g instanceof e"));
    assert_eq!("/a/i instanceof e", minify("/a/i instanceof e"));
}

#[test]
fn test_remove_extra_whitespace_before_typeof() {
    let source = "var x = typeof 'foo';var y = typeof x;case typeof 'foo': 'bla'";

    let expected_result = "var x=typeof'foo';var y=typeof x;case typeof'foo':'bla'";
    assert_eq!(minify(source), expected_result);
}

#[test]
fn test_remove_extra_whitespace_before_in() {
    let source = r#"if ("key" in ev && typeof ev) { return true; }
if (x in ev && typeof ev) { return true; }
if (true in ev) { return true; }"#;

    let expected_result = r#"if("key"in ev&&typeof ev){return true}if(x in ev&&typeof ev){return true}if(true in ev){return true}"#;
    assert_eq!(minify(source), expected_result);
}

#[test]
fn test_remove_extra_whitespace_before_operator() {
    let source = "( x ) / 2; x / y;x /= y";

    let expected_result = "(x)/2;x/y;x/=y";
    assert_eq!(minify(source), expected_result);
}

#[test]
fn check_regex_syntax() {
    let source = "console.log(/MSIE|Trident|Edge/.test(window.navigator.userAgent));";
    let expected = "console.log(/MSIE|Trident|Edge/.test(window.navigator.userAgent))";
    assert_eq!(minify(source), expected);
}

#[test]
fn minify_minified() {
    let source = "function (i, n, a) { i[n].type.replace(/ *;(.|\\s)*/,\"\")===t&&a.push(i[n].MathJax.elementJax);return a}";
    let expected = "function(i,n,a){i[n].type.replace(/ *;(.|\\s)*/,\"\")===t&&a.push(i[n].MathJax.elementJax);return a}";
    assert_eq!(minify(source), expected);
}

#[test]
fn check_string() {
    let source = r###"
        const a = 123;
        const b = "123";
        const c = `the number is ${a}  <-- note the spaces here`;
        const d = `      ${a}         ${b}      `;
    "###;
    let expected = "const a=123;const b=\"123\";const c=`the number is ${a}  <-- note the spaces \
    here`;const d=`      ${a}         ${b}      `";
    assert_eq!(minify(source), expected);
}

// TODO: requires AST to fix this issue!
/*#[test]
fn no_semi_colon() {
    let source = r#"
console.log(1)
console.log(2)
var x = 12;
"#;
    let expected_result = r#"console.log(1);console.log(2);var x=12;"#;
    assert_eq!(minify(source), expected_result);
}*/

// TODO: requires AST to fix this issue!
/*#[test]
fn correct_replace_for_backline() {
    let source = r#"
function foo() {
    return
    12;
}
"#;
    let expected_result = r#"function foo(){return 12;}"#;
    assert_eq!(minify(source), expected_result);
}*/
