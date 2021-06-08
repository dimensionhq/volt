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

mod token;
mod tools;
mod utils;

pub use self::token::{tokenize, Condition, Keyword, Operation, ReservedChar, Token, Tokens};
pub use self::tools::{
    aggregate_strings, aggregate_strings_into_array, aggregate_strings_into_array_filter,
    aggregate_strings_into_array_with_separation,
    aggregate_strings_into_array_with_separation_filter, aggregate_strings_with_separation, minify,
    simple_minify,
};
pub use self::utils::{
    clean_token, clean_token_except, clean_tokens, clean_tokens_except,
    get_variable_name_and_value_positions, replace_token_with, replace_tokens_with,
};
