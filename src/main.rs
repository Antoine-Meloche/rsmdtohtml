use std::fs::File;
use std::io::{ BufRead, BufReader };
use regex::Regex;

fn main() {
    let file_path = std::env::args().nth(1).expect("Missing parameter file_path");

    let file_contents = open_file(&file_path);

    let html = parse_file(file_contents);

    for line in html {
        println!("{}", line)
    }
}

fn open_file(file_path: &str) -> Vec<String> {
    let file = File::open(file_path).expect(format!("Could not open file `{}`", file_path).as_str());
    let reader = BufReader::new(file);

    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();

    return lines;
}

fn parse_file(content: Vec<String>) -> Vec<String> {
    let mut new_lines: Vec<String> = Vec::new();
    let mut in_code_block: bool = false;

    for line in content {
        let mut html_line = String::from(line.clone());

        if line.trim().starts_with("```") {
            in_code_block = !in_code_block;
            new_lines.push(String::from(if in_code_block { "<pre><code>" } else { "</code></pre>" }));
            continue
        } else if in_code_block {
            new_lines.push(line);
            continue
        } else if line.trim() == "" {
            new_lines.push(String::from("<br />"));
            continue;
        }

        html_line = parse_headers(html_line);
        html_line = parse_list(html_line); // TODO: Add lists support both ordered and unordered
        html_line = parse_bold_italics(html_line);
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

fn parse_links(line: String) -> String { // TODO: Add lists support
    let mut new_line: String = line;
    return new_line;
}

fn parse_strike(line: String) -> String {
    let mut new_line: String = line;

    let strike_re = Regex::new(r"~~(.+?)~~").unwrap();
    new_line = strike_re.replace_all(&new_line, "<s>$1</s>").to_string();

    return new_line;
}
