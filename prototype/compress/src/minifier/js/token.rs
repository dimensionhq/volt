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
use std::convert::TryFrom;
use std::fmt;
use std::str::{CharIndices, FromStr};

use macro_utils::if_match;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum ReservedChar {
    Comma,
    OpenParenthese,
    CloseParenthese,
    OpenCurlyBrace,
    CloseCurlyBrace,
    OpenBracket,
    CloseBracket,
    Colon,
    SemiColon,
    Dot,
    Quote,
    DoubleQuote,
    ExclamationMark,
    QuestionMark,
    Slash,
    Modulo,
    Star,
    Minus,
    Plus,
    EqualSign,
    Backslash,
    Space,
    Tab,
    Backline,
    LessThan,
    SuperiorThan,
    Pipe,
    Ampersand,
    BackTick,
}

impl ReservedChar {
    pub fn is_white_character(&self) -> bool {
        *self == ReservedChar::Space
            || *self == ReservedChar::Tab
            || *self == ReservedChar::Backline
    }
}

impl fmt::Display for ReservedChar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                ReservedChar::Comma => ',',
                ReservedChar::OpenParenthese => '(',
                ReservedChar::CloseParenthese => ')',
                ReservedChar::OpenCurlyBrace => '{',
                ReservedChar::CloseCurlyBrace => '}',
                ReservedChar::OpenBracket => '[',
                ReservedChar::CloseBracket => ']',
                ReservedChar::Colon => ':',
                ReservedChar::SemiColon => ';',
                ReservedChar::Dot => '.',
                ReservedChar::Quote => '\'',
                ReservedChar::DoubleQuote => '"',
                ReservedChar::ExclamationMark => '!',
                ReservedChar::QuestionMark => '?',
                ReservedChar::Slash => '/',
                ReservedChar::Modulo => '%',
                ReservedChar::Star => '*',
                ReservedChar::Minus => '-',
                ReservedChar::Plus => '+',
                ReservedChar::EqualSign => '=',
                ReservedChar::Backslash => '\\',
                ReservedChar::Space => ' ',
                ReservedChar::Tab => '\t',
                ReservedChar::Backline => '\n',
                ReservedChar::LessThan => '<',
                ReservedChar::SuperiorThan => '>',
                ReservedChar::Pipe => '|',
                ReservedChar::Ampersand => '&',
                ReservedChar::BackTick => '`',
            }
        )
    }
}

impl TryFrom<char> for ReservedChar {
    type Error = &'static str;

    fn try_from(value: char) -> Result<ReservedChar, Self::Error> {
        match value {
            ',' => Ok(ReservedChar::Comma),
            '(' => Ok(ReservedChar::OpenParenthese),
            ')' => Ok(ReservedChar::CloseParenthese),
            '{' => Ok(ReservedChar::OpenCurlyBrace),
            '}' => Ok(ReservedChar::CloseCurlyBrace),
            '[' => Ok(ReservedChar::OpenBracket),
            ']' => Ok(ReservedChar::CloseBracket),
            ':' => Ok(ReservedChar::Colon),
            ';' => Ok(ReservedChar::SemiColon),
            '.' => Ok(ReservedChar::Dot),
            '\'' => Ok(ReservedChar::Quote),
            '"' => Ok(ReservedChar::DoubleQuote),
            '!' => Ok(ReservedChar::ExclamationMark),
            '?' => Ok(ReservedChar::QuestionMark),
            '/' => Ok(ReservedChar::Slash),
            '%' => Ok(ReservedChar::Modulo),
            '*' => Ok(ReservedChar::Star),
            '-' => Ok(ReservedChar::Minus),
            '+' => Ok(ReservedChar::Plus),
            '=' => Ok(ReservedChar::EqualSign),
            '\\' => Ok(ReservedChar::Backslash),
            ' ' => Ok(ReservedChar::Space),
            '\t' => Ok(ReservedChar::Tab),
            '\n' | '\r' => Ok(ReservedChar::Backline),
            '<' => Ok(ReservedChar::LessThan),
            '>' => Ok(ReservedChar::SuperiorThan),
            '|' => Ok(ReservedChar::Pipe),
            '&' => Ok(ReservedChar::Ampersand),
            '`' => Ok(ReservedChar::BackTick),
            _ => Err("Unknown reserved char"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Keyword {
    Break,
    Case,
    Catch,
    Const,
    Continue,
    Default,
    Do,
    Else,
    False,
    Finally,
    Function,
    For,
    If,
    In,
    InstanceOf,
    Let,
    New,
    Null,
    Private,
    Protected,
    Public,
    Return,
    Switch,
    This,
    Throw,
    True,
    Try,
    Typeof,
    Static,
    Var,
    While,
}

impl Keyword {
    fn requires_before(&self) -> bool {
        matches!(*self, Keyword::In | Keyword::InstanceOf)
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Keyword::Break => "break",
                Keyword::Case => "case",
                Keyword::Catch => "catch",
                Keyword::Const => "const",
                Keyword::Continue => "continue",
                Keyword::Default => "default",
                Keyword::Do => "do",
                Keyword::Else => "else",
                Keyword::False => "false",
                Keyword::Finally => "finally",
                Keyword::Function => "function",
                Keyword::For => "for",
                Keyword::If => "if",
                Keyword::In => "in",
                Keyword::InstanceOf => "instanceof",
                Keyword::Let => "let",
                Keyword::New => "new",
                Keyword::Null => "null",
                Keyword::Private => "private",
                Keyword::Protected => "protected",
                Keyword::Public => "public",
                Keyword::Return => "return",
                Keyword::Switch => "switch",
                Keyword::This => "this",
                Keyword::Throw => "throw",
                Keyword::True => "true",
                Keyword::Try => "try",
                Keyword::Typeof => "typeof",
                Keyword::Static => "static",
                Keyword::Var => "var",
                Keyword::While => "while",
            }
        )
    }
}

impl<'a> TryFrom<&'a str> for Keyword {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Keyword, Self::Error> {
        match value {
            "break" => Ok(Keyword::Break),
            "case" => Ok(Keyword::Case),
            "catch" => Ok(Keyword::Catch),
            "const" => Ok(Keyword::Const),
            "continue" => Ok(Keyword::Continue),
            "default" => Ok(Keyword::Default),
            "do" => Ok(Keyword::Do),
            "else" => Ok(Keyword::Else),
            "false" => Ok(Keyword::False),
            "finally" => Ok(Keyword::Finally),
            "function" => Ok(Keyword::Function),
            "for" => Ok(Keyword::For),
            "if" => Ok(Keyword::If),
            "in" => Ok(Keyword::In),
            "instanceof" => Ok(Keyword::InstanceOf),
            "let" => Ok(Keyword::Let),
            "new" => Ok(Keyword::New),
            "null" => Ok(Keyword::Null),
            "private" => Ok(Keyword::Private),
            "protected" => Ok(Keyword::Protected),
            "public" => Ok(Keyword::Public),
            "return" => Ok(Keyword::Return),
            "switch" => Ok(Keyword::Switch),
            "this" => Ok(Keyword::This),
            "throw" => Ok(Keyword::Throw),
            "true" => Ok(Keyword::True),
            "try" => Ok(Keyword::Try),
            "typeof" => Ok(Keyword::Typeof),
            "static" => Ok(Keyword::Static),
            "var" => Ok(Keyword::Var),
            "while" => Ok(Keyword::While),
            _ => Err("Unkown keyword"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum Condition {
    And,
    Or,
    DifferentThan,
    SuperDifferentThan,
    EqualTo,
    SuperEqualTo,
    SuperiorThan,
    SuperiorOrEqualTo,
    InferiorThan,
    InferiorOrEqualTo,
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Condition::And => "&&",
                Condition::Or => "||",
                Condition::DifferentThan => "!=",
                Condition::SuperDifferentThan => "!==",
                Condition::EqualTo => "==",
                Condition::SuperEqualTo => "===",
                Condition::SuperiorThan => ">",
                Condition::SuperiorOrEqualTo => ">=",
                Condition::InferiorThan => "<",
                Condition::InferiorOrEqualTo => "<=",
            }
        )
    }
}

impl TryFrom<ReservedChar> for Condition {
    type Error = &'static str;

    fn try_from(value: ReservedChar) -> Result<Condition, Self::Error> {
        Ok(match value {
            ReservedChar::SuperiorThan => Condition::SuperiorThan,
            ReservedChar::LessThan => Condition::InferiorThan,
            _ => return Err("Unkown condition"),
        })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum Operation {
    Addition,
    AdditionEqual,
    Subtract,
    SubtractEqual,
    Multiply,
    MultiplyEqual,
    Divide,
    DivideEqual,
    Modulo,
    ModuloEqual,
    Equal,
}

impl Operation {
    pub fn is_assign(&self) -> bool {
        matches!(
            *self,
            Operation::AdditionEqual
                | Operation::SubtractEqual
                | Operation::MultiplyEqual
                | Operation::DivideEqual
                | Operation::ModuloEqual
                | Operation::Equal
        )
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Operation::Addition => "+",
                Operation::AdditionEqual => "+=",
                Operation::Subtract => "-",
                Operation::SubtractEqual => "-=",
                Operation::Multiply => "*",
                Operation::MultiplyEqual => "*=",
                Operation::Divide => "/",
                Operation::DivideEqual => "/=",
                Operation::Modulo => "%",
                Operation::ModuloEqual => "%=",
                Operation::Equal => "=",
            }
        )
    }
}

impl TryFrom<ReservedChar> for Operation {
    type Error = &'static str;

    fn try_from(value: ReservedChar) -> Result<Operation, Self::Error> {
        Ok(match value {
            ReservedChar::Plus => Operation::Addition,
            ReservedChar::Minus => Operation::Subtract,
            ReservedChar::Slash => Operation::Divide,
            ReservedChar::Star => Operation::Multiply,
            ReservedChar::Modulo => Operation::Modulo,
            ReservedChar::EqualSign => Operation::Equal,
            _ => return Err("Unkown operation"),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub enum Token<'a> {
    Keyword(Keyword),
    Char(ReservedChar),
    String(&'a str),
    Comment(&'a str),
    License(&'a str),
    Other(&'a str),
    Regex {
        regex: &'a str,
        is_global: bool,
        is_interactive: bool,
    },
    Condition(Condition),
    Operation(Operation),
    CreatedVarDecl(String),
    CreatedVar(String),
    Number(usize),
    FloatingNumber(&'a str),
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Token::Keyword(x) => write!(f, "{}", x),
            Token::Char(x) => write!(f, "{}", x),
            Token::String(x) | Token::Comment(x) | Token::Other(x) => write!(f, "{}", x),
            Token::License(x) => write!(f, "/*!{}*/", x),
            Token::Regex {
                regex,
                is_global,
                is_interactive,
            } => {
                let x = write!(f, "/{}/", regex);
                if is_global {
                    write!(f, "g")?;
                }
                if is_interactive {
                    write!(f, "i")?;
                }
                x
            }
            Token::Condition(x) => write!(f, "{}", x),
            Token::Operation(x) => write!(f, "{}", x),
            Token::CreatedVarDecl(ref x) => write!(f, "{}", x),
            Token::CreatedVar(ref x) => write!(f, "{}", x),
            Token::Number(x) => write!(f, "{}", x),
            Token::FloatingNumber(ref x) => write!(f, "{}", x),
        }
    }
}

impl<'a> Token<'a> {
    pub fn is_comment(&self) -> bool {
        matches!(*self, Token::Comment(_))
    }

    pub fn is_license(&self) -> bool {
        matches!(*self, Token::License(_))
    }

    pub fn is_reserved_char(&self) -> bool {
        matches!(*self, Token::Char(_))
    }

    pub fn get_char(&self) -> Option<ReservedChar> {
        match *self {
            Token::Char(c) => Some(c),
            _ => None,
        }
    }

    pub fn eq_char(&self, rc: ReservedChar) -> bool {
        match *self {
            Token::Char(c) => c == rc,
            _ => false,
        }
    }

    pub fn eq_operation(&self, ope: Operation) -> bool {
        match *self {
            Token::Operation(o) => o == ope,
            _ => false,
        }
    }

    pub fn is_operation(&self) -> bool {
        matches!(*self, Token::Operation(_))
    }

    pub fn eq_condition(&self, cond: Condition) -> bool {
        match *self {
            Token::Condition(c) => c == cond,
            _ => false,
        }
    }

    pub fn is_condition(&self) -> bool {
        matches!(*self, Token::Condition(_))
    }

    pub fn is_other(&self) -> bool {
        matches!(*self, Token::Other(_))
    }

    pub fn get_other(&self) -> Option<&str> {
        match *self {
            Token::Other(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_white_character(&self) -> bool {
        match *self {
            Token::Char(c) => c.is_white_character(),
            _ => false,
        }
    }

    pub fn is_keyword(&self) -> bool {
        matches!(*self, Token::Keyword(_))
    }

    pub fn get_keyword(&self) -> Option<Keyword> {
        match *self {
            Token::Keyword(k) => Some(k),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(*self, Token::String(_))
    }

    pub fn get_string(&self) -> Option<&str> {
        match *self {
            Token::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_regex(&self) -> bool {
        matches!(*self, Token::Regex { .. })
    }

    pub fn is_created_var_decl(&self) -> bool {
        matches!(*self, Token::CreatedVarDecl(_))
    }

    pub fn is_created_var(&self) -> bool {
        matches!(*self, Token::CreatedVar(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(*self, Token::Number(_))
    }

    pub fn is_floating_number(&self) -> bool {
        matches!(*self, Token::FloatingNumber(_))
    }

    fn get_required(&self) -> Option<char> {
        match *self {
            Token::Keyword(_)
            | Token::Other(_)
            | Token::CreatedVarDecl(_)
            | Token::Number(_)
            | Token::FloatingNumber(_) => Some(' '),
            _ => None,
        }
    }

    fn requires_before(&self) -> bool {
        match *self {
            Token::Keyword(k) => k.requires_before(),
            _ => false,
        }
    }
}

fn get_line_comment<'a>(
    source: &'a str,
    iterator: &mut MyPeekable<'_>,
    start_pos: &mut usize,
) -> Option<Token<'a>> {
    *start_pos += 1;
    for (pos, c) in iterator {
        if let Ok(c) = ReservedChar::try_from(c) {
            if c == ReservedChar::Backline {
                let ret = Some(Token::Comment(&source[*start_pos..pos]));
                *start_pos = pos;
                return ret;
            }
        }
    }
    None
}

fn get_regex<'a>(
    source: &'a str,
    iterator: &mut MyPeekable<'_>,
    start_pos: &mut usize,
    v: &[Token],
) -> Option<Token<'a>> {
    let mut back = v.len();
    while back > 0 {
        back -= 1;
        if v[back].is_white_character() || v[back].is_comment() || v[back].is_license() {
            continue;
        }
        match &v[back] {
            Token::Char(ReservedChar::SemiColon)
            | Token::Char(ReservedChar::Colon)
            | Token::Char(ReservedChar::Comma)
            | Token::Char(ReservedChar::OpenBracket)
            | Token::Char(ReservedChar::OpenParenthese)
            | Token::Operation(Operation::Equal) => break,
            _ => return None,
        }
    }
    iterator.start_save();
    while let Some((pos, c)) = iterator.next() {
        if c == '\\' {
            // we skip next character
            iterator.next();
            continue;
        }
        if let Ok(c) = ReservedChar::try_from(c) {
            if c == ReservedChar::Slash {
                let mut is_global = false;
                let mut is_interactive = false;
                let mut add = 0;
                loop {
                    match iterator.peek() {
                        Some((_, 'i')) => is_interactive = true,
                        Some((_, 'g')) => is_global = true,
                        _ => break,
                    };
                    iterator.next();
                    add += 1;
                }
                let ret = Some(Token::Regex {
                    regex: &source[*start_pos + 1..pos],
                    is_interactive,
                    is_global,
                });
                *start_pos = pos + add;
                iterator.drop_save();
                return ret;
            } else if c == ReservedChar::Backline {
                break;
            }
        }
    }
    iterator.stop_save();
    None
}

fn get_comment<'a>(
    source: &'a str,
    iterator: &mut MyPeekable<'_>,
    start_pos: &mut usize,
) -> Option<Token<'a>> {
    let mut prev = ReservedChar::Quote;
    *start_pos += 1;
    let builder = if let Some((_, c)) = iterator.next() {
        if c == '!' {
            *start_pos += 1;
            Token::License
        } else {
            if let Ok(c) = ReservedChar::try_from(c) {
                prev = c;
            }
            Token::Comment
        }
    } else {
        Token::Comment
    };

    for (pos, c) in iterator {
        if let Ok(c) = ReservedChar::try_from(c) {
            if c == ReservedChar::Slash && prev == ReservedChar::Star {
                let ret = Some(builder(&source[*start_pos..pos - 1]));
                *start_pos = pos;
                return ret;
            }
            prev = c;
        } else {
            prev = ReservedChar::Space;
        }
    }
    None
}

fn get_string<'a>(
    source: &'a str,
    iterator: &mut MyPeekable<'_>,
    start_pos: &mut usize,
    start: ReservedChar,
) -> Option<Token<'a>> {
    while let Some((pos, c)) = iterator.next() {
        if c == '\\' {
            // we skip next character
            iterator.next();
            continue;
        }
        if let Ok(c) = ReservedChar::try_from(c) {
            if c == start {
                let ret = Some(Token::String(&source[*start_pos..pos + 1]));
                *start_pos = pos;
                return ret;
            }
        }
    }
    None
}

fn get_backtick_string<'a>(
    source: &'a str,
    iterator: &mut MyPeekable<'_>,
    start_pos: &mut usize,
) -> Option<Token<'a>> {
    while let Some((pos, c)) = iterator.next() {
        if c == '\\' {
            // we skip next character
            iterator.next();
            continue;
        }
        if c == '$' && iterator.peek().map(|(_, c)| c == '{').unwrap_or(false) {
            let mut count = 0;

            loop {
                if let Some((mut pos, c)) = iterator.next() {
                    if c == '\\' {
                        // we skip next character
                        iterator.next();
                        continue;
                    } else if c == '"' || c == '\'' {
                        // We don't care about the result
                        get_string(
                            source,
                            iterator,
                            &mut pos,
                            ReservedChar::try_from(c)
                                .expect("ReservedChar::try_from unexpectedly failed..."),
                        );
                    } else if c == '`' {
                        get_backtick_string(source, iterator, &mut pos);
                    } else if c == '{' {
                        count += 1;
                    } else if c == '}' {
                        count -= 1;
                        if count == 0 {
                            break;
                        }
                    }
                } else {
                    return None;
                }
            }
        } else if c == '`' {
            let ret = Some(Token::String(&source[*start_pos..pos + 1]));
            *start_pos = pos;
            return ret;
        }
    }
    None
}

fn first_useful<'a>(v: &'a [Token<'a>]) -> Option<&'a Token<'a>> {
    for x in v.iter().rev() {
        if x.is_white_character() {
            continue;
        }
        return Some(x);
    }
    None
}

fn fill_other<'a>(source: &'a str, v: &mut Vec<Token<'a>>, start: usize, pos: usize) {
    if start < pos {
        if let Ok(w) = Keyword::try_from(&source[start..pos]) {
            v.push(Token::Keyword(w));
        } else if let Ok(n) = usize::from_str(&source[start..pos]) {
            v.push(Token::Number(n))
        } else if f64::from_str(&source[start..pos]).is_ok() {
            v.push(Token::FloatingNumber(&source[start..pos]))
        } else {
            v.push(Token::Other(&source[start..pos]));
        }
    }
}

fn handle_equal_sign(v: &mut Vec<Token>, c: ReservedChar) -> bool {
    if c != ReservedChar::EqualSign {
        return false;
    }
    if_match! {
        v.last().unwrap_or(&Token::Other("")).eq_operation(Operation::Equal) => {
            v.pop();
            v.push(Token::Condition(Condition::EqualTo));
        },
        v.last().unwrap_or(&Token::Other("")).eq_condition(Condition::EqualTo) => {
            v.pop();
            v.push(Token::Condition(Condition::SuperEqualTo));
        },
        v.last().unwrap_or(&Token::Other("")).eq_char(ReservedChar::ExclamationMark) => {
            v.pop();
            v.push(Token::Condition(Condition::DifferentThan));
        },
        v.last().unwrap_or(&Token::Other("")).eq_condition(Condition::DifferentThan) => {
            v.pop();
            v.push(Token::Condition(Condition::SuperDifferentThan));
        },
        v.last().unwrap_or(&Token::Other("")).eq_operation(Operation::Divide) => {
            v.pop();
            v.push(Token::Operation(Operation::DivideEqual));
        },
        v.last().unwrap_or(&Token::Other("")).eq_operation(Operation::Multiply) => {
            v.pop();
            v.push(Token::Operation(Operation::MultiplyEqual));
        },
        v.last().unwrap_or(&Token::Other("")).eq_operation(Operation::Addition) => {
            v.pop();
            v.push(Token::Operation(Operation::AdditionEqual));
        },
        v.last().unwrap_or(&Token::Other("")).eq_operation(Operation::Subtract) => {
            v.pop();
            v.push(Token::Operation(Operation::SubtractEqual));
        },
        v.last().unwrap_or(&Token::Other("")).eq_operation(Operation::Modulo) => {
            v.pop();
            v.push(Token::Operation(Operation::ModuloEqual));
        },
        v.last().unwrap_or(&Token::Other("")).eq_condition(Condition::SuperiorThan) => {
            v.pop();
            v.push(Token::Condition(Condition::SuperiorOrEqualTo));
        },
        v.last().unwrap_or(&Token::Other("")).eq_condition(Condition::InferiorThan) => {
            v.pop();
            v.push(Token::Condition(Condition::InferiorOrEqualTo));
        },
        else => {
            return false;
        }
    }
    true
}

fn check_if_number<'a>(
    iterator: &mut MyPeekable,
    start: usize,
    pos: usize,
    source: &'a str,
) -> bool {
    if source[start..pos].find('.').is_some() {
        return false;
    } else if u64::from_str(&source[start..pos]).is_ok() {
        return true;
    } else if let Some((_, x)) = iterator.peek() {
        return x as u8 >= b'0' && x as u8 <= b'9';
    }
    false
}

struct MyPeekable<'a> {
    inner: CharIndices<'a>,
    saved: Vec<(usize, char)>,
    peeked: Option<(usize, char)>,
    is_saving: bool,
}

impl<'a> MyPeekable<'a> {
    fn new(indices: CharIndices<'a>) -> MyPeekable<'a> {
        MyPeekable {
            inner: indices,
            saved: Vec::with_capacity(500),
            peeked: None,
            is_saving: false,
        }
    }

    fn start_save(&mut self) {
        self.is_saving = true;
        if let Some(p) = self.peeked {
            self.saved.push(p);
        }
    }

    fn drop_save(&mut self) {
        self.is_saving = false;
        self.saved.clear();
    }

    fn stop_save(&mut self) {
        self.is_saving = false;
        if let Some(p) = self.peeked {
            self.saved.push(p);
        }
        self.peeked = None;
    }

    /// Returns None if saving.
    fn peek(&mut self) -> Option<(usize, char)> {
        if self.peeked.is_none() {
            self.peeked = self.inner.next();
            if self.is_saving {
                if let Some(p) = self.peeked {
                    self.saved.push(p);
                }
            }
        }
        self.peeked
    }
}

impl<'a> Iterator for MyPeekable<'a> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        if self.peeked.is_some() {
            self.peeked.take()
        } else {
            if !self.is_saving && !self.saved.is_empty() {
                return Some(self.saved.remove(0));
            }
            match self.inner.next() {
                Some(r) if self.is_saving => {
                    self.saved.push(r);
                    Some(r)
                }
                r => r,
            }
        }
    }
}

pub fn tokenize(source: &str) -> Tokens<'_> {
    let mut v = Vec::with_capacity(1000);
    let mut start = 0;
    let mut iterator = MyPeekable::new(source.char_indices());

    loop {
        let (mut pos, c) = match iterator.next() {
            Some(x) => x,
            None => {
                fill_other(source, &mut v, start, source.len());
                break;
            }
        };
        if let Ok(c) = ReservedChar::try_from(c) {
            if c == ReservedChar::Dot && check_if_number(&mut iterator, start, pos, source) {
                let mut cont = true;
                if let Some(x) = iterator.peek() {
                    if !"0123456789,; \t\n<>/*&|{}[]-+=~%^:!".contains(x.1) {
                        fill_other(source, &mut v, start, pos);
                        start = pos;
                        cont = false;
                    }
                }
                if cont {
                    continue;
                }
            }
            fill_other(source, &mut v, start, pos);
            if_match! {
                c == ReservedChar::Quote || c == ReservedChar::DoubleQuote =>
                    if let Some(s) = get_string(source, &mut iterator, &mut pos, c) {
                        v.push(s);
                    },
                c == ReservedChar::BackTick =>
                    if let Some(s) = get_backtick_string(source, &mut iterator, &mut pos) {
                        v.push(s);
                    },
                c == ReservedChar::Slash &&
                v.last().unwrap_or(&Token::Other("")).eq_operation(Operation::Divide) => {
                    v.pop();
                    if let Some(s) = get_line_comment(source, &mut iterator, &mut pos) {
                        v.push(s);
                    }
                },
                c == ReservedChar::Slash &&
                iterator.peek().is_some() &&
                iterator.peek().unwrap().1 != '/' &&
                iterator.peek().unwrap().1 != '*' &&
                !first_useful(&v).unwrap_or(&Token::String("")).is_other() => {
                    if let Some(r) = get_regex(source, &mut iterator, &mut pos, &v) {
                        v.push(r);
                    } else {
                        v.push(Token::Operation(Operation::Divide));
                    }
                },
                c == ReservedChar::Star &&
                v.last().unwrap_or(&Token::Other("")).eq_operation(Operation::Divide) => {
                    v.pop();
                    if let Some(s) = get_comment(source, &mut iterator, &mut pos) {
                        v.push(s);
                    }
                },
                c == ReservedChar::Pipe &&
                v.last().unwrap_or(&Token::Other("")).eq_char(ReservedChar::Pipe) => {
                    v.pop();
                    v.push(Token::Condition(Condition::Or));
                },
                c == ReservedChar::Ampersand &&
                v.last().unwrap_or(&Token::Other("")).eq_char(ReservedChar::Ampersand) => {
                    v.pop();
                    v.push(Token::Condition(Condition::And));
                },
                handle_equal_sign(&mut v, c) => {},
                let Ok(o) = Operation::try_from(c) => {
                    v.push(Token::Operation(o));
                },
                let Ok(o) = Condition::try_from(c) => {
                    v.push(Token::Condition(o));
                },
                else => {
                    v.push(Token::Char(c));
                }
            }
            start = pos + 1;
        }
    }
    Tokens(v)
}

#[derive(Debug, PartialEq)]
pub struct Tokens<'a>(pub Vec<Token<'a>>);

impl<'a> fmt::Display for Tokens<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tokens = &self.0;
        for i in 0..tokens.len() {
            if i > 0
                && tokens[i].requires_before()
                && !tokens[i - 1].is_keyword()
                && !tokens[i - 1].is_other()
                && !tokens[i - 1].is_reserved_char()
                && !tokens[i - 1].is_string()
            {
                write!(f, " ")?;
            }
            write!(f, "{}", tokens[i])?;
            if let Some(c) = match tokens[i] {
                Token::Keyword(_) | Token::Other(_) if i + 1 < tokens.len() => {
                    tokens[i + 1].get_required()
                }
                _ => None,
            } {
                write!(f, "{}", c)?;
            }
        }
        Ok(())
    }
}

impl<'a> Tokens<'a> {
    pub fn apply<F>(self, func: F) -> Tokens<'a>
    where
        F: Fn(Tokens<'a>) -> Tokens<'a>,
    {
        func(self)
    }
}

pub struct IntoIterTokens<'a> {
    inner: Tokens<'a>,
}

impl<'a> IntoIterator for Tokens<'a> {
    type Item = (Token<'a>, Option<&'a Token<'a>>);
    type IntoIter = IntoIterTokens<'a>;

    fn into_iter(mut self) -> Self::IntoIter {
        self.0.reverse();
        IntoIterTokens { inner: self }
    }
}

impl<'a> Iterator for IntoIterTokens<'a> {
    type Item = (Token<'a>, Option<&'a Token<'a>>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.0.is_empty() {
            None
        } else {
            let ret = self.inner.0.pop().expect("pop() failed");
            // FIXME once generic traits' types are stabilized, use a second
            // lifetime instead of transmute!
            Some((ret, unsafe { ::std::mem::transmute(self.inner.0.last()) }))
        }
    }
}

impl<'a> ::std::ops::Deref for Tokens<'a> {
    type Target = Vec<Token<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<Vec<Token<'a>>> for Tokens<'a> {
    fn from(v: Vec<Token<'a>>) -> Self {
        Tokens(v)
    }
}

impl<'a> From<&[Token<'a>]> for Tokens<'a> {
    fn from(v: &[Token<'a>]) -> Self {
        Tokens(v.to_vec())
    }
}

#[test]
fn check_regex() {
    let source = r#"var x = /"\.x/g;"#;
    let expected_result = r#"var x=/"\.x/g"#;
    assert_eq!(super::minify(source), expected_result);

    let v = tokenize(source).apply(super::clean_tokens);
    assert_eq!(
        v.0[3],
        Token::Regex {
            regex: "\"\\.x",
            is_global: true,
            is_interactive: false,
        }
    );

    let source = r#"var x = /"\.x/gigigigig;var x = "hello";"#;
    let expected_result = r#"var x=/"\.x/gi;var x="hello""#;
    assert_eq!(super::minify(source), expected_result);

    let v = tokenize(source).apply(super::clean_tokens);
    assert_eq!(
        v.0[3],
        Token::Regex {
            regex: "\"\\.x",
            is_global: true,
            is_interactive: true,
        }
    );
}

#[test]
fn more_regex() {
    let source = r#"var x = /"\.x\/a/i;"#;
    let expected_result = r#"var x=/"\.x\/a/i"#;
    assert_eq!(super::minify(source), expected_result);

    let v = tokenize(source).apply(super::clean_tokens);
    assert_eq!(
        v.0[3],
        Token::Regex {
            regex: "\"\\.x\\/a",
            is_global: false,
            is_interactive: true,
        }
    );

    let source = r#"var x = /\\/i;"#;
    let expected_result = r#"var x=/\\/i"#;
    assert_eq!(super::minify(source), expected_result);

    let v = tokenize(source).apply(super::clean_tokens);
    assert_eq!(
        v.0[3],
        Token::Regex {
            regex: "\\\\",
            is_global: false,
            is_interactive: true,
        }
    );
}

#[test]
fn even_more_regex() {
    let source = r#"var x = /a-z /;"#;

    let v = tokenize(source).apply(super::clean_tokens);
    assert_eq!(
        v.0[3],
        Token::Regex {
            regex: "a-z ",
            is_global: false,
            is_interactive: false,
        }
    );
}

#[test]
fn not_regex_test() {
    let source = "( x ) / 2; x / y;x /= y";

    let v = tokenize(source).apply(super::clean_tokens);
    assert_eq!(
        &v.0,
        &[
            Token::Char(ReservedChar::OpenParenthese),
            Token::Other("x"),
            Token::Char(ReservedChar::CloseParenthese),
            Token::Operation(Operation::Divide),
            Token::Number(2),
            Token::Char(ReservedChar::SemiColon),
            Token::Other("x"),
            Token::Operation(Operation::Divide),
            Token::Other("y"),
            Token::Char(ReservedChar::SemiColon),
            Token::Other("x"),
            Token::Operation(Operation::DivideEqual),
            Token::Other("y")
        ]
    );

    let source = "let x = /x\ny/;";

    let v = tokenize(source).apply(super::clean_tokens);
    assert_eq!(
        &v.0,
        &[
            Token::Keyword(Keyword::Let),
            Token::Other("x"),
            Token::Operation(Operation::Equal),
            Token::Operation(Operation::Divide),
            Token::Other("x"),
            Token::Other("y"),
            Token::Operation(Operation::Divide)
        ]
    );
}

#[test]
fn test_tokens_parsing() {
    let source = "true = == 2.3 === 32";

    let v = tokenize(source).apply(super::clean_tokens);
    assert_eq!(
        &v.0,
        &[
            Token::Keyword(Keyword::True),
            Token::Operation(Operation::Equal),
            Token::Condition(Condition::EqualTo),
            Token::FloatingNumber("2.3"),
            Token::Condition(Condition::SuperEqualTo),
            Token::Number(32)
        ]
    );
}

#[test]
fn test_string_parsing() {
    let source = "var x = 'hello people!'";

    let v = tokenize(source).apply(super::clean_tokens);
    assert_eq!(
        &v.0,
        &[
            Token::Keyword(Keyword::Var),
            Token::Other("x"),
            Token::Operation(Operation::Equal),
            Token::String("\'hello people!\'")
        ]
    );
}

#[test]
fn test_number_parsing() {
    let source = "var x = .12; let y = 4.; var z = 12; .3 4. 'a' let u = 12.2";

    let v = tokenize(source).apply(super::clean_tokens);
    assert_eq!(
        &v.0,
        &[
            Token::Keyword(Keyword::Var),
            Token::Other("x"),
            Token::Operation(Operation::Equal),
            Token::FloatingNumber(".12"),
            Token::Char(ReservedChar::SemiColon),
            Token::Keyword(Keyword::Let),
            Token::Other("y"),
            Token::Operation(Operation::Equal),
            Token::FloatingNumber("4."),
            Token::Char(ReservedChar::SemiColon),
            Token::Keyword(Keyword::Var),
            Token::Other("z"),
            Token::Operation(Operation::Equal),
            Token::Number(12),
            Token::Char(ReservedChar::SemiColon),
            Token::FloatingNumber(".3"),
            Token::FloatingNumber("4."),
            Token::String("'a'"),
            Token::Keyword(Keyword::Let),
            Token::Other("u"),
            Token::Operation(Operation::Equal),
            Token::FloatingNumber("12.2")
        ]
    );
}

#[test]
fn test_number_parsing2() {
    let source = "var x = 12.a;";

    let v = tokenize(source).apply(super::clean_tokens);
    assert_eq!(
        &v.0,
        &[
            Token::Keyword(Keyword::Var),
            Token::Other("x"),
            Token::Operation(Operation::Equal),
            Token::Number(12),
            Token::Char(ReservedChar::Dot),
            Token::Other("a")
        ]
    );
}

#[test]
fn tokens_spaces() {
    let source = "t in e";

    let v = tokenize(source).apply(super::clean_tokens);
    assert_eq!(
        &v.0,
        &[
            Token::Other("t"),
            Token::Keyword(Keyword::In),
            Token::Other("e")
        ]
    );
}

#[test]
fn division_by_id() {
    let source = "100/abc";

    let v = tokenize(source).apply(super::clean_tokens);
    assert_eq!(
        &v.0,
        &[
            Token::Number(100),
            Token::Operation(Operation::Divide),
            Token::Other("abc")
        ]
    );
}
