use std::fs::File;
use std::io::{ BufRead, BufReader, BufWriter, Write };
use regex::Regex;

fn main() {
    let file_path: String = std::env::args().nth(1).expect("Missing parameter file_path");
    let output_path: String = std::env::args().nth(2).expect("Missing parameter output_path");

    let file_contents: Vec<String> = open_file(&file_path);

    println!("Markdown file `{}` has been opened.", file_path);

    let html: Vec<String> = parse_file(file_contents);

    println!("Markdown has been parsed into HTML.");

    write_file(&output_path, html).unwrap();

    println!("HTML has been written to `{}`.", output_path);
}

fn open_file(file_path: &str) -> Vec<String> {
    let file = File::open(file_path).expect(format!("Could not open file `{}`", file_path).as_str());
    let reader = BufReader::new(file);

    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();

    return lines;
}

fn write_file(file_path: &str, content: Vec<String>) -> std::io::Result<()> {
    let file = File::create(file_path).expect(format!("Could not create file `{}`", file_path).as_str());
    let mut writer = BufWriter::new(file);

    for line in content {
        writer.write_all(line.as_bytes()).unwrap();
        writer.write_all(b"\n").unwrap();
    }

    writer.flush().unwrap();

    return Ok(());
}

fn parse_file(content: Vec<String>) -> Vec<String> {
    let mut new_lines: Vec<String> = Vec::new();
    let mut in_code_block: bool = false;
    let mut in_list: bool = false;
    let list_re = Regex::new(r"\s*(\*|-|(\d\.)) (.+)").unwrap();
    let mut indent_level = 0;
    let mut list_type: String;
    let mut list_hist: Vec<String> = Vec::new();

    for line in content {
        let mut html_line = String::from(line.clone());

        if line.trim().starts_with("```") {
            in_code_block = !in_code_block;
            new_lines.push(String::from(if in_code_block { "<pre><code>" } else { "</code></pre>" }));
            continue
        }

        if in_code_block {
            new_lines.push(line);
            continue
        }

        if in_list {
            if !list_re.is_match(&line) {
                in_list = false;
                close_lists(&mut list_hist, &mut new_lines);
            }
        }

        html_line = parse_bold_italics(html_line);

        if list_re.is_match(&line) {
            in_list = true;

            let current_indent = line.chars().take_while(|c| *c == ' ').count()/4;

            if current_indent > indent_level {
                if line.trim().starts_with('*') || line.trim().starts_with('-') {
                    list_type = String::from("<ul>");
                } else {
                    list_type = String::from("<ol>");
                }

                list_hist.push(format!("</{}", &list_type[1..]));
                new_lines.push(list_type);
            } else if current_indent < indent_level {
                new_lines.push(list_hist.last().unwrap().to_string());
                list_hist.pop();
            }

            indent_level = current_indent;

            html_line = parse_lists(html_line);
        }

        if line.trim() == "" {
            new_lines.push(String::from("<br />"));
            continue;
        }

        html_line = parse_headers(html_line);
        html_line = parse_code(html_line);
        html_line = parse_separator(html_line);
        html_line = parse_images(html_line);
        html_line = parse_links(html_line);
        html_line = parse_strike(html_line);

        new_lines.push(html_line);
    };

    return new_lines;
}

fn parse_headers(line: String) -> String {
    let header_level = line.chars().take_while(|&c| c == '#').count();
    let header_text = line.trim_start_matches('#').trim();

    return match header_level {
        1 => format!("<h1>{}</h1>", header_text),
        2 => format!("<h2>{}</h2>", header_text),
        3 => format!("<h3>{}</h3>", header_text),
        4 => format!("<h4>{}</h4>", header_text),
        5 => format!("<h5>{}</h5>", header_text),
        6 => format!("<h6>{}</h6>", header_text),
        _ => line
    }
}

fn parse_bold_italics(line: String) -> String {
    let mut new_line: String = line;

    let boldtalic_re = Regex::new(r"\*\*\*(.+?)\*\*\*").unwrap();
    new_line = boldtalic_re.replace_all(&new_line, "<b><i>$1</i></b>").to_string();
    
    let bold_re = Regex::new(r"\*\*(.+?)\*\*").unwrap();
    new_line = bold_re.replace_all(&new_line, "<b>$1</b>").to_string();

    let italic_re = Regex::new(r"\*(.+?)\*").unwrap();
    new_line = italic_re.replace_all(&new_line, "<i>$1</i>").to_string();

    return new_line;
}

fn parse_code(line: String) -> String {
    let mut new_line: String = line;

    let singleline_code_re = Regex::new(r"\`(.+?)\`").unwrap();
    new_line = singleline_code_re.replace_all(&new_line, "<code>$1</code>").to_string();

    return new_line;
}

fn parse_separator(line: String) -> String {
    let separator_re = Regex::new(r"^-{3}-*\s*$").unwrap();
    
    if separator_re.is_match(&line) {
        return String::from("<hr>");
    } else {
        return line;
    }
}

fn parse_images(line: String) -> String {
    let mut new_line: String = line;

    let image_re = Regex::new(r"!\[(.*?)\]\((.+?)\)").unwrap();
    new_line = image_re.replace_all(&new_line, "<img alt=\"$1\" src=\"$2\">").to_string();

    return new_line;
}

fn parse_links(line: String) -> String {
    let mut new_line: String = line;
    
    let link_re = Regex::new(r"\[(.*?)\]\((.+?)\)").unwrap();
    new_line = link_re.replace_all(&new_line, "<a href=\"$2\">$1</a>").to_string();

    return new_line;
}

fn parse_strike(line: String) -> String {
    let mut new_line: String = line;

    let strike_re = Regex::new(r"~~(.+?)~~").unwrap();
    new_line = strike_re.replace_all(&new_line, "<s>$1</s>").to_string();

    return new_line;
}

fn parse_lists(line: String) -> String {
    let mut new_line: String = line.trim().to_string();

    let list_re = Regex::new(r"\s*(?:\*|-|(?:\d\.)) (.+)").unwrap();
    new_line = list_re.replace_all(&new_line, "<li>$1</li>").to_string();

    return new_line;
}

fn close_lists(list_hist: &mut Vec<String>, new_lines: &mut Vec<String>) {
    while !list_hist.is_empty() {
        new_lines.push(list_hist.last().unwrap().to_string());
        list_hist.pop();
    }
}
