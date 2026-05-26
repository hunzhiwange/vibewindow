//! 承载 SQL 工具的格式化、持久化与临时状态逻辑。
//!
//! 注释说明当前文件的职责边界，帮助调用方理解数据流与错误传播，
//! 不改变任何运行时行为。

/// 执行 purify_sql 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn purify_sql(input: &str) -> Option<String> {
    let sql = strip_comments(input);
    let mut out_lines: Vec<String> = Vec::new();

    for line in sql.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        out_lines.push(trimmed.to_string());
    }

    if out_lines.is_empty() { None } else { Some(out_lines.join("\n")) }
}

/// 执行 compress_sql 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn compress_sql(input: &str) -> Option<String> {
    let sql = strip_comments(input);
    let mut out = String::with_capacity(sql.len());
    let mut prev_was_space = false;
    let mut in_single = false;
    let mut in_double = false;
    let mut in_backtick = false;
    let mut in_bracket = false;

    let mut it = sql.chars().peekable();
    while let Some(ch) = it.next() {
        if in_single {
            out.push(ch);
            if ch == '\'' {
                if it.peek() == Some(&'\'') {
                    out.push('\'');
                    let _ = it.next();
                } else {
                    in_single = false;
                }
            }
            continue;
        }

        if in_double {
            out.push(ch);
            if ch == '"' {
                if it.peek() == Some(&'"') {
                    out.push('"');
                    let _ = it.next();
                } else {
                    in_double = false;
                }
            }
            continue;
        }

        if in_backtick {
            out.push(ch);
            if ch == '`' {
                in_backtick = false;
            }
            continue;
        }

        if in_bracket {
            out.push(ch);
            if ch == ']' {
                in_bracket = false;
            }
            continue;
        }

        match ch {
            '\'' => {
                in_single = true;
                out.push(ch);
                prev_was_space = false;
            }
            '"' => {
                in_double = true;
                out.push(ch);
                prev_was_space = false;
            }
            '`' => {
                in_backtick = true;
                out.push(ch);
                prev_was_space = false;
            }
            '[' => {
                in_bracket = true;
                out.push(ch);
                prev_was_space = false;
            }
            ch if ch.is_whitespace() => {
                if !prev_was_space {
                    out.push(' ');
                    prev_was_space = true;
                }
            }
            ',' | ';' | ')' => {
                if out.ends_with(' ') {
                    out.pop();
                }
                out.push(ch);
                out.push(' ');
                prev_was_space = true;
            }
            '(' => {
                if out.ends_with(' ') {
                    out.pop();
                }
                out.push('(');
                prev_was_space = false;
            }
            _ => {
                out.push(ch);
                prev_was_space = false;
            }
        }
    }

    let sql = out.trim().to_string();
    if sql.is_empty() { None } else { Some(sql) }
}

/// 执行 beautify_sql 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn beautify_sql(input: &str) -> Option<String> {
    let purified = strip_comments(input);
    let tokens = tokenize_sql(&purified);
    let mut out = String::new();
    let mut depth = 0usize;
    let mut line_has_content = false;
    let mut clause = Clause::Other;
    let mut pending_space = false;

    let mut i = 0usize;
    while i < tokens.len() {
        let token = &tokens[i];
        match token {
            Token::Whitespace => {
                pending_space = line_has_content;
            }
            Token::Symbol(symbol) => match symbol.as_str() {
                "(" => {
                    if pending_space && !ends_with_space_or_newline(&out) {
                        out.push(' ');
                    }
                    out.push('(');
                    pending_space = false;
                    line_has_content = true;
                    depth = depth.saturating_add(1);

                    let next_non_whitespace =
                        tokens.iter().skip(i + 1).find(|token| !matches!(token, Token::Whitespace));
                    if let Some(Token::Word(word)) = next_non_whitespace {
                        if is_major_clause(word) || word.eq_ignore_ascii_case("select") {
                            newline(&mut out, depth, &mut line_has_content);
                        }
                    } else if let Some(Token::Symbol(next_symbol)) = next_non_whitespace {
                        if next_symbol != ")" {
                            newline(&mut out, depth, &mut line_has_content);
                        }
                    }
                }
                ")" => {
                    depth = depth.saturating_sub(1);
                    if line_has_content && !out.ends_with('\n') {
                        newline(&mut out, depth, &mut line_has_content);
                    }
                    out.push_str(&"    ".repeat(depth));
                    out.push(')');
                    pending_space = false;
                    line_has_content = true;
                }
                "," => {
                    out.push(',');
                    pending_space = false;
                    line_has_content = true;
                    if clause == Clause::SelectList {
                        newline(&mut out, depth + 1, &mut line_has_content);
                    } else {
                        pending_space = true;
                    }
                }
                ";" => {
                    if out.ends_with(' ') {
                        out.pop();
                    }
                    out.push(';');
                    pending_space = false;
                    line_has_content = true;
                }
                _ => {
                    if pending_space && !ends_with_space_or_newline(&out) {
                        out.push(' ');
                    }
                    out.push_str(symbol);
                    pending_space = false;
                    line_has_content = true;
                }
            },
            Token::StringLiteral(string_literal) => {
                if pending_space && !ends_with_space_or_newline(&out) {
                    out.push(' ');
                }
                out.push_str(string_literal);
                pending_space = false;
                line_has_content = true;
            }
            Token::Word(word) => {
                let upper = uppercase_keyword(word);
                let lower = word.to_ascii_lowercase();
                let prev_word = previous_non_whitespace_word(&tokens, i);
                let next_word = next_non_whitespace_word(&tokens, i);

                if should_newline_before(&lower, clause, prev_word, next_word) {
                    newline(&mut out, depth, &mut line_has_content);
                }

                if lower == "select" {
                    clause = Clause::SelectList;
                } else if lower == "where" || lower == "having" {
                    clause = Clause::Where;
                } else if lower == "on" || lower == "using" {
                    clause = Clause::On;
                } else if is_clause_boundary(&lower) {
                    clause = Clause::Other;
                }

                if (lower == "and" || lower == "or") && matches!(clause, Clause::Where | Clause::On)
                {
                    newline(&mut out, depth + 1, &mut line_has_content);
                }

                if pending_space && !ends_with_space_or_newline(&out) {
                    out.push(' ');
                }
                out.push_str(&upper);
                pending_space = true;
                line_has_content = true;

                if lower == "delete" || lower == "insert" || lower == "update" || lower == "select"
                {
                    if !out.ends_with('\n') && !out.ends_with(' ') {
                        out.push(' ');
                    }
                    pending_space = false;
                }

                if lower == "group" || lower == "order" {
                    let next_non_whitespace =
                        tokens.iter().skip(i + 1).find(|token| !matches!(token, Token::Whitespace));
                    if matches!(next_non_whitespace, Some(Token::Word(next)) if next.eq_ignore_ascii_case("by")) {
                        out.push(' ');
                        pending_space = false;
                    }
                }
            }
        }
        i += 1;
    }

    let sql = out.trim().to_string();
    if sql.is_empty() { None } else { Some(sql + "\n") }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Clause {
    SelectList,
    Where,
    On,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    StringLiteral(String),
    Symbol(String),
    Whitespace,
}

fn tokenize_sql(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut it = input.chars().peekable();
    while let Some(ch) = it.peek().copied() {
        if ch.is_whitespace() {
            while matches!(it.peek(), Some(current) if current.is_whitespace()) {
                let _ = it.next();
            }
            tokens.push(Token::Whitespace);
            continue;
        }

        if ch == '\'' || ch == '"' || ch == '`' {
            let quote = ch;
            let mut string_literal = String::new();
            string_literal.push(quote);
            let _ = it.next();
            while let Some(current) = it.next() {
                string_literal.push(current);
                if current == quote {
                    if (quote == '\'' || quote == '"') && it.peek() == Some(&quote) {
                        string_literal.push(quote);
                        let _ = it.next();
                        continue;
                    }
                    break;
                }
            }
            tokens.push(Token::StringLiteral(string_literal));
            continue;
        }

        if ch == '[' {
            let mut identifier = String::new();
            identifier.push('[');
            let _ = it.next();
            for current in it.by_ref() {
                identifier.push(current);
                if current == ']' {
                    break;
                }
            }
            tokens.push(Token::StringLiteral(identifier));
            continue;
        }

        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
            let mut word = String::new();
            while let Some(current) = it.peek().copied() {
                if current.is_ascii_alphanumeric()
                    || current == '_'
                    || current == '.'
                    || current == '$'
                {
                    word.push(current);
                    let _ = it.next();
                } else {
                    break;
                }
            }
            tokens.push(Token::Word(word));
            continue;
        }

        let Some(symbol) = it.next() else {
            break;
        };
        tokens.push(Token::Symbol(symbol.to_string()));
    }
    tokens
}

fn newline(out: &mut String, depth: usize, line_has_content: &mut bool) {
    if out.ends_with('\n') {
        out.push_str(&"    ".repeat(depth));
        *line_has_content = false;
        return;
    }
    out.push('\n');
    out.push_str(&"    ".repeat(depth));
    *line_has_content = false;
}

fn ends_with_space_or_newline(input: &str) -> bool {
    matches!(input.chars().last(), Some(' ') | Some('\n'))
}

fn is_major_clause(word_lower: &str) -> bool {
    matches!(
        word_lower,
        "with"
            | "select"
            | "from"
            | "where"
            | "group"
            | "order"
            | "having"
            | "limit"
            | "offset"
            | "union"
            | "values"
            | "set"
            | "insert"
            | "update"
            | "delete"
            | "join"
            | "returning"
            | "window"
    )
}

fn is_clause_boundary(word_lower: &str) -> bool {
    matches!(
        word_lower,
        "with"
            | "from"
            | "where"
            | "group"
            | "order"
            | "having"
            | "limit"
            | "offset"
            | "union"
            | "values"
            | "set"
            | "returning"
            | "window"
    )
}

fn should_newline_before(
    word_lower: &str,
    clause: Clause,
    prev_word: Option<&str>,
    next_word: Option<&str>,
) -> bool {
    match word_lower {
        "from" => matches!(clause, Clause::SelectList),
        "with" | "where" | "group" | "order" | "having" | "limit" | "offset" | "union"
        | "returning" | "window" => true,
        "join" => !prev_word.is_some_and(is_join_modifier),
        "inner" | "left" | "right" | "full" | "cross" => next_word.is_some_and(|word| {
            word.eq_ignore_ascii_case("join") || word.eq_ignore_ascii_case("outer")
        }),
        "on" | "using" | "set" | "values" => true,
        _ => false,
    }
}

fn previous_non_whitespace_word(tokens: &[Token], index: usize) -> Option<&str> {
    for token in tokens[..index].iter().rev() {
        match token {
            Token::Whitespace => continue,
            Token::Word(word) => return Some(word.as_str()),
            _ => return None,
        }
    }

    None
}

fn next_non_whitespace_word(tokens: &[Token], index: usize) -> Option<&str> {
    for token in tokens.iter().skip(index + 1) {
        match token {
            Token::Whitespace => continue,
            Token::Word(word) => return Some(word.as_str()),
            _ => return None,
        }
    }

    None
}

fn is_join_modifier(word: &str) -> bool {
    matches!(
        word,
        current if current.eq_ignore_ascii_case("inner")
            || current.eq_ignore_ascii_case("left")
            || current.eq_ignore_ascii_case("right")
            || current.eq_ignore_ascii_case("full")
            || current.eq_ignore_ascii_case("cross")
            || current.eq_ignore_ascii_case("outer")
    )
}

fn uppercase_keyword(word: &str) -> String {
    let lower = word.to_ascii_lowercase();
    if is_keyword(&lower) { lower.to_ascii_uppercase() } else { word.to_string() }
}

fn is_keyword(word_lower: &str) -> bool {
    matches!(
        word_lower,
        "with"
            | "recursive"
            | "select"
            | "from"
            | "where"
            | "group"
            | "by"
            | "order"
            | "having"
            | "limit"
            | "offset"
            | "insert"
            | "into"
            | "values"
            | "update"
            | "set"
            | "delete"
            | "and"
            | "or"
            | "not"
            | "exists"
            | "in"
            | "is"
            | "null"
            | "like"
            | "join"
            | "inner"
            | "left"
            | "right"
            | "full"
            | "cross"
            | "outer"
            | "on"
            | "using"
            | "union"
            | "all"
            | "distinct"
            | "as"
            | "case"
            | "when"
            | "then"
            | "else"
            | "end"
            | "between"
            | "returning"
            | "window"
            | "over"
            | "partition"
            | "filter"
            | "asc"
            | "desc"
    )
}

fn strip_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut it = input.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    let mut in_backtick = false;
    let mut in_bracket = false;

    while let Some(ch) = it.next() {
        if in_single {
            out.push(ch);
            if ch == '\'' {
                if it.peek() == Some(&'\'') {
                    out.push('\'');
                    let _ = it.next();
                } else {
                    in_single = false;
                }
            }
            continue;
        }

        if in_double {
            out.push(ch);
            if ch == '"' {
                if it.peek() == Some(&'"') {
                    out.push('"');
                    let _ = it.next();
                } else {
                    in_double = false;
                }
            }
            continue;
        }

        if in_backtick {
            out.push(ch);
            if ch == '`' {
                in_backtick = false;
            }
            continue;
        }

        if in_bracket {
            out.push(ch);
            if ch == ']' {
                in_bracket = false;
            }
            continue;
        }

        match ch {
            '\'' => {
                in_single = true;
                out.push(ch);
            }
            '"' => {
                in_double = true;
                out.push(ch);
            }
            '`' => {
                in_backtick = true;
                out.push(ch);
            }
            '[' => {
                in_bracket = true;
                out.push(ch);
            }
            '-' => {
                if it.peek() == Some(&'-') {
                    let _ = it.next();
                    for current in it.by_ref() {
                        if current == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                } else {
                    out.push(ch);
                }
            }
            '/' => {
                if it.peek() == Some(&'*') {
                    let _ = it.next();
                    let mut prev = '\0';
                    for current in it.by_ref() {
                        if prev == '*' && current == '/' {
                            break;
                        }
                        if current == '\n' {
                            out.push('\n');
                        }
                        prev = current;
                    }
                } else {
                    out.push(ch);
                }
            }
            _ => out.push(ch),
        }
    }
    out
}
#[cfg(test)]
#[path = "formatting_tests.rs"]
mod formatting_tests;
