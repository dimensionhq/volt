// MIT License
//
// Copyright (c) 2017 Guillaume Gomez
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

use regex::{Captures, Regex};

fn condense_whitespace(source: &str) -> String {
    let lower_source = source.to_lowercase();
    if lower_source.find("<textarea").is_none() && lower_source.find("<pre").is_none() {
        // maybe should be better not to recreate Regex every time?
        let re = Regex::new(r">\s+<").unwrap();
        let source = re.replace_all(source, "> <").into_owned();
        let re = Regex::new(r"\s{2,}|[\r\n]").unwrap();
        re.replace_all(&source, " ").into_owned()
    } else {
        source.trim().to_owned()
    }
}

fn condense(source: &str) -> String {
    let re = Regex::new(r"<(style|script)[\w|\s].*?>").unwrap();
    let type_reg = Regex::new(r#"\s*?type="[\w|\s].*?""#).unwrap();
    re.replace_all(source, |caps: &Captures| {
        type_reg.replace_all(&caps[0], "").into_owned()
    })
    .into_owned()
}

fn clean_unneeded_tags(source: &str) -> String {
    let useless_tags = [
        "</area>",
        "</base>",
        "<body>",
        "</body>",
        "</br>",
        "</col>",
        "</colgroup>",
        "</dd>",
        "</dt>",
        "<head>",
        "</head>",
        "</hr>",
        "<html>",
        "</html>",
        "</img>",
        "</input>",
        "</li>",
        "</link>",
        "</meta>",
        "</option>",
        "</param>",
        "<tbody>",
        "</tbody>",
        "</td>",
        "</tfoot>",
        "</th>",
        "</thead>",
        "</tr>",
        "</basefont>",
        "</isindex>",
        "</param>",
    ];
    let mut res = source.to_owned();
    for useless_tag in &useless_tags {
        res = res.replace(useless_tag, "");
    }
    res
}

fn remove_comments(source: &str) -> String {
    // "build" and "endbuild" should be matched case insensitively.
    let re = Regex::new("<!--(.|\n)*?-->").unwrap();
    re.replace_all(source, |caps: &Captures| {
        if caps[0].replace("<!--", " ").trim().starts_with("[") {
            caps[0].to_owned()
        } else {
            " ".to_owned()
        }
    })
    .into_owned()
}

fn unquote_attributes(source: &str) -> String {
    // Some attributes like width, height, etc... don't need quotes.
    let any_tag = Regex::new(r"<\w.*?>").unwrap();
    let extra_spaces = Regex::new(r" \s+|\s +").unwrap();
    let between_words = Regex::new(r"\w\s+\w").unwrap();
    let spaces_before_close = Regex::new(r##""\s+>"##).unwrap();
    let spaces_before_close2 = Regex::new(r"'\s+>").unwrap();
    let extra_spaces2 = Regex::new(r##""\s\s+\w+="|'\s\s+\w+='|"\s\s+\w+=|'\s\s+\w+="##).unwrap();
    let extra_spaces3 = Regex::new(r"\d\s+>").unwrap();
    let quotes_in_tag = Regex::new(r##"([a-zA-Z]+)="([a-zA-Z0-9-_\.]+)""##).unwrap();

    any_tag
        .replace_all(source, |caps: &Captures| {
            let cap = format!("{}", &caps[0]);
            if cap.starts_with("<!") || cap.find("</").is_some() {
                cap
            } else {
                let tag = spaces_before_close.replace_all(&cap, "\">").into_owned();
                let mut tag = spaces_before_close2.replace_all(&tag, "'>").into_owned();
                let tag_c = tag.clone();

                let space1_matches: Vec<_> = between_words.find_iter(&tag_c).collect();
                let space6_matches: Vec<_> = extra_spaces3.find_iter(&tag_c).collect();
                let mut pos = 0;
                loop {
                    let replacement = match (space1_matches.get(pos), space6_matches.get(pos)) {
                        (Some(a), Some(b)) => format!("{}{}", a.as_str(), b.as_str()),
                        (None, Some(b)) => format!("{}", b.as_str()),
                        (Some(a), None) => format!("{}", a.as_str()),
                        _ => break,
                    };
                    pos += 1;
                    tag = tag.replace(
                        &replacement,
                        &extra_spaces.replace_all(&replacement, " ").into_owned(),
                    );
                }
                let mut output = tag.clone();
                for caps in extra_spaces2.find_iter(&tag) {
                    let c = caps.as_str().chars().next().unwrap_or('\0');
                    output = output.replace(
                        caps.as_str(),
                        &format!(
                            "{} {}",
                            if c == '\0' {
                                String::new()
                            } else {
                                format!("{}", c)
                            },
                            caps.as_str()[1..].trim_start()
                        ),
                    );
                }
                tag = quotes_in_tag
                    .replace_all(&output, |caps: &Captures| match &caps[1] {
                        "width" | "height" => format!("{}={}", &caps[1], &caps[2]),
                        x => format!("{}=\"{}\"", x, &caps[2]),
                    })
                    .into_owned();
                if cap != tag {
                    tag
                } else {
                    cap
                }
            }
        })
        .trim()
        .to_owned()
}

/// Returns a minified version of the provided HTML source.
pub fn minify(source: &str) -> String {
    let source = remove_comments(source);
    let source = condense(&source);
    let source = clean_unneeded_tags(&source);
    let source = condense_whitespace(&source);
    unquote_attributes(&source).trim().to_owned()
}

#[test]
fn html_minify_test() {
    let source = r##"<head>
    <title>Some huge title</title>
    <link rel="stylesheet" type="text/css"   href="something.css"   >
    <style type="text/css">
        .some_class {
            color: red;
        }
    </style>
</head>
<body>
    <header>
        <div>
            <i>    <b><a href="www.somewhere.com" class="some_class">Narnia</a> </b>    </i>
            <h1    style="width:100%;text-align:center;"   >Big header</h1>
        </div>
    <!-- commeeeeeeeents !!! -->
    </header>
    <div id="some_id">
        <!-- another comment
        on
multi
lines -->
        <div id="another_id" class="another_class" width="100">
            <h2>A little sub title</h2>
            <ul>
                <li>A list!</li>
                <li>Who doesn't like lists?</li>
                <li height="12" class="fooool">Well, who cares...</li>
            </ul>
        </div>
    </div>
    <script type="text/javascript"    >
        console.log("foo");
    </script>
    <style type="text/css" src="../foo.css">
    <script src="../foo.js">
</body>
"##;

    let expected_result = "<title>Some huge title</title> <link rel=\"stylesheet\" \
                           type=\"text/css\" href=\"something.css\"> <style> .some_class \
                           { color: red; } </style> <header> <div> <i> <b><a \
                           href=\"www.somewhere.com\" class=\"some_class\">Narnia</a> </b> </i> \
                           <h1 style=\"width:100%;text-align:center;\">Big header</h1> </div> \
                           </header> <div id=\"some_id\"> <div id=\"another_id\" \
                           class=\"another_class\" width=100> <h2>A little sub \
                           title</h2> <ul> <li>A list! <li>Who doesn't like lists? \
                           <li height=12 class=\"fooool\">Well, who cares... </ul> </div> \
                           </div> <script > console.log(\"foo\"); </script> <style \
                           src=\"../foo.css\"> <script src=\"../foo.js\">";
    assert_eq!(minify(source), expected_result);
}

#[test]
fn html_keep_important_comments() {
    let source = r#"
<div>
    <!-- normal comment -->
    <div>content</div>
    <!--[if lte IE 8]>
    <div class="warning">This old browser is unsupported and will most likely display funky things.
    </div>
    <![endif]-->
</div>
"#;

    let expected_result =
        "<div> <div>content</div> <!--[if lte IE 8]> <div class=\"warning\">This \
                           old browser is unsupported and will most likely display funky things. \
                           </div> <![endif]--> </div>";
    assert_eq!(minify(source), expected_result);
}
