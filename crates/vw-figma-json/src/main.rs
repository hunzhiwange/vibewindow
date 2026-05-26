use anyhow::{Context, Result, anyhow, bail};
use std::fs;
use std::path::{Path, PathBuf};

const HELP: &str = "\
Convert Figma .fig files to JSON

Usage:
  fig2json <input.fig> [-o output.json] [--compact] [-v] [--raw]
  fig2json <input.zip> <extract-dir> [--compact] [-v] [--raw]

Options:
  -o, --output <path>  Output JSON file path
      --compact        Emit compact JSON instead of pretty output
  -v, --verbose        Print progress information
      --raw            Also emit untransformed raw JSON
  -h, --help           Show this help
  -V, --version        Show version";

struct Cli {
    input: PathBuf,
    extract_dir: Option<PathBuf>,
    output: Option<PathBuf>,
    compact: bool,
    verbose: bool,
    raw: bool,
}

fn main() -> Result<()> {
    let cli = parse_cli()?;

    if cli.verbose {
        eprintln!("Reading input file: {}", cli.input.display());
    }

    // 读取输入文件
    let bytes = fs::read(&cli.input)
        .with_context(|| format!("Failed to read input file: {}", cli.input.display()))?;

    if cli.verbose {
        eprintln!("File size: {} bytes", bytes.len());
    }

    // 检查输入是否是 ZIP 容器
    let is_zip = fig2json::parser::is_zip_container(&bytes);

    // 根据文件类型验证参数
    if is_zip {
        // ZIP模式：需要extract_dir，禁止-o
        let extract_dir = cli.extract_dir.as_ref().ok_or_else(|| {
            anyhow!("ZIP files require an extraction directory as second argument")
        })?;

        if cli.output.is_some() {
            bail!("Cannot use -o/--output flag with extraction directory (ZIP mode)");
        }

        // ZIP 提取模式
        handle_zip_mode(&bytes, extract_dir, cli.compact, cli.verbose, cli.raw)?;
    } else {
        // 常规 .fig 文件模式
        if cli.verbose {
            eprintln!("Converting to JSON...");
        }

        // 确定图像文件操作的基目录
        let base_dir = if let Some(output_path) = &cli.output {
            output_path.parent()
        } else {
            // 如果输出到标准输出，则使用当前目录
            Some(Path::new("."))
        };

        let json =
            fig2json::convert(&bytes, base_dir).context("Failed to convert .fig file to JSON")?;

        if cli.verbose {
            eprintln!("Conversion successful!");
        }

        // 格式化输出(默认情况下很漂亮，如果设置了标志则很紧凑)
        let output = if cli.compact {
            serde_json::to_string(&json)?
        } else {
            serde_json::to_string_pretty(&json)?
        };

        // 写输出
        match cli.output.as_ref() {
            Some(path) => {
                if cli.verbose {
                    eprintln!("Writing output to: {}", path.display());
                }
                fs::write(path, &output)
                    .with_context(|| format!("Failed to write output file: {}", path.display()))?;
                if cli.verbose {
                    eprintln!("Done!");
                }
            }
            None => {
                println!("{}", output);
            }
        }

        // 如果设置了 --raw 标志，还会生成原始 JSON 文件
        if cli.raw {
            if cli.verbose {
                eprintln!("Converting to raw JSON...");
            }

            let raw_json =
                fig2json::convert_raw(&bytes).context("Failed to convert .fig file to raw JSON")?;

            let raw_output = if cli.compact {
                serde_json::to_string(&raw_json)?
            } else {
                serde_json::to_string_pretty(&raw_json)?
            };

            // 确定原始输出路径
            let raw_path = match cli.output.as_ref() {
                Some(path) => {
                    // 从输出路径派生 .raw.json
                    let mut raw = path.clone();
                    raw.set_extension("raw.json");
                    raw
                }
                None => {
                    // 从输入路径派生
                    cli.input.with_extension("raw.json")
                }
            };

            if cli.verbose {
                eprintln!("Writing raw output to: {}", raw_path.display());
            }

            fs::write(&raw_path, raw_output).with_context(|| {
                format!("Failed to write raw output file: {}", raw_path.display())
            })?;

            if cli.verbose {
                eprintln!("Raw JSON done!");
            }
        }
    }

    Ok(())
}

fn parse_cli() -> Result<Cli> {
    let mut args = std::env::args_os().skip(1);
    let mut positionals = Vec::new();
    let mut output = None;
    let mut compact = false;
    let mut verbose = false;
    let mut raw = false;

    while let Some(arg) = args.next() {
        let arg_str = arg.to_string_lossy();
        match arg_str.as_ref() {
            "-o" | "--output" => {
                let value = args.next().ok_or_else(|| anyhow!("Missing value for {}", arg_str))?;
                output = Some(PathBuf::from(value));
            }
            "--compact" => compact = true,
            "-v" | "--verbose" => verbose = true,
            "--raw" => raw = true,
            "-h" | "--help" => {
                println!("{HELP}");
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            value if value.starts_with('-') => bail!("Unknown option: {}", value),
            _ => positionals.push(PathBuf::from(arg)),
        }
    }

    if positionals.is_empty() {
        bail!("{HELP}");
    }

    if positionals.len() > 2 {
        bail!("Too many positional arguments");
    }

    Ok(Cli {
        input: positionals.remove(0),
        extract_dir: positionals.pop(),
        output,
        compact,
        verbose,
        raw,
    })
}

/// 处理 ZIP 提取模式：提取所有文件并转换找到的所有 .fig 文件
fn handle_zip_mode(
    zip_bytes: &[u8],
    extract_dir: &PathBuf,
    compact: bool,
    verbose: bool,
    raw: bool,
) -> Result<()> {
    if verbose {
        eprintln!("ZIP file detected - extracting to: {}", extract_dir.display());
    }

    // 将整个 ZIP 解压缩到目录
    fig2json::parser::extract_zip_to_directory(zip_bytes, extract_dir)
        .context("Failed to extract ZIP file")?;

    if verbose {
        eprintln!("ZIP extracted successfully");
        eprintln!("Searching for .fig files...");
    }

    // 查找提取内容中的所有 .fig 文件
    let fig_files = find_fig_files(extract_dir)?;

    if fig_files.is_empty() {
        bail!("No .fig files found in ZIP archive");
    }

    let file_count = fig_files.len();

    if verbose {
        eprintln!("Found {} .fig file(s)", file_count);
    }

    // 转换每个 .fig 文件
    for fig_path in fig_files {
        let relative_path = fig_path.strip_prefix(extract_dir).unwrap_or(&fig_path);

        if verbose {
            eprintln!("Converting: {}", relative_path.display());
        }

        // 读取.fig文件
        let fig_bytes = fs::read(&fig_path)
            .with_context(|| format!("Failed to read .fig file: {}", fig_path.display()))?;

        // 确定图像文件操作的基目录(.fig 文件的父目录)
        let base_dir = fig_path.parent();

        // 转换为 JSON
        let json = fig2json::convert(&fig_bytes, base_dir)
            .with_context(|| format!("Failed to convert: {}", fig_path.display()))?;

        // 格式化输出(默认情况下很漂亮，如果设置了标志则很紧凑)
        let output = if compact {
            serde_json::to_string(&json)?
        } else {
            serde_json::to_string_pretty(&json)?
        };

        // 确定输出路径：与 .fig 相同，但扩展名为 .json
        let output_path = fig_path.with_extension("json");

        // 写入 JSON 文件
        fs::write(&output_path, output)
            .with_context(|| format!("Failed to write output: {}", output_path.display()))?;

        if verbose {
            eprintln!(
                "  → {}",
                output_path.strip_prefix(extract_dir).unwrap_or(&output_path).display()
            );
        }

        // 如果设置了 --raw 标志，还会生成原始 JSON 文件
        if raw {
            let raw_json = fig2json::convert_raw(&fig_bytes).with_context(|| {
                format!("Failed to convert to raw JSON: {}", fig_path.display())
            })?;

            let raw_output = if compact {
                serde_json::to_string(&raw_json)?
            } else {
                serde_json::to_string_pretty(&raw_json)?
            };

            // 确定原始输出路径：与 .fig 相同，但扩展名为 .raw.json
            let raw_output_path = fig_path.with_extension("raw.json");

            // 写入原始 JSON 文件
            fs::write(&raw_output_path, raw_output).with_context(|| {
                format!("Failed to write raw output: {}", raw_output_path.display())
            })?;

            if verbose {
                eprintln!(
                    "  → {}",
                    raw_output_path.strip_prefix(extract_dir).unwrap_or(&raw_output_path).display()
                );
            }
        }
    }

    if verbose {
        eprintln!("Done! Converted {} file(s)", file_count);
    }

    Ok(())
}

/// 递归查找目录中的所有 .fig 文件
fn find_fig_files(dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut fig_files = Vec::new();

    fn visit_dir(dir: &PathBuf, fig_files: &mut Vec<PathBuf>) -> Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    visit_dir(&path, fig_files)?;
                } else if path.extension().and_then(|s| s.to_str()) == Some("fig") {
                    fig_files.push(path);
                }
            }
        }
        Ok(())
    }

    visit_dir(dir, &mut fig_files)?;
    Ok(fig_files)
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod main_tests;
