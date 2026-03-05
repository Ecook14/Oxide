// ============================================================
// oxidec — The Oxide Language Compiler
// ============================================================
// Entry point for the Oxide compiler frontend.
//
// Usage:
//   oxidec lex     <file.ox>  — Tokenize and dump tokens
//   oxidec parse   <file.ox>  — Parse and dump AST
//   oxidec check   <file.ox>  — Semantic analysis (ownership, regions, types)
//   oxidec compile <file.ox>  — Full pipeline: Lex → Parse → Analyze → OxIR
// ============================================================

mod token;
mod ast;
mod lexer;
mod parser;
mod analyzer;
mod codegen;
mod validator;
mod backend;

use std::env;
use std::fs;
use std::process;
use std::collections::HashSet;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("╔══════════════════════════════════════════════════╗");
        eprintln!("║           oxidec — Oxide Compiler v0.1          ║");
        eprintln!("╚══════════════════════════════════════════════════╝");
        eprintln!();
        eprintln!("Usage: oxidec <command> <file.ox>");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  lex      — Tokenize and print tokens");
        eprintln!("  parse    — Parse and print AST");
        eprintln!("  check    — Run semantic analysis");
        eprintln!("  compile  — Full pipeline → OxIR output");
        process::exit(1);
    }

    let command = &args[1];
    let filename = &args[2];

    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", filename, e);
            process::exit(1);
        }
    };

    match command.as_str() {
        "lex"     => run_lexer(&source),
        "parse"   => run_parser(&source, filename),
        "check"   => run_checker(&source, filename),
        "compile" => run_compile(&source, filename),
        "build"   => run_build(&source, filename),
        _ => {
            eprintln!("Unknown command: '{}'. Use 'lex', 'parse', 'check', 'compile', or 'build'.", command);
            process::exit(1);
        }
    }
}

fn run_lexer(source: &str) {
    let mut lex = lexer::Lexer::new(source);
    match lex.tokenize() {
        Ok(tokens) => {
            println!("╔══════════════════════════════════════════════════╗");
            println!("║          OXIDE COMPILER — LEXER OUTPUT          ║");
            println!("╚══════════════════════════════════════════════════╝");
            println!();
            for tok in &tokens {
                println!(
                    "  [{:>3}:{:<3}]  {:20} │ \"{}\"",
                    tok.span.line, tok.span.column,
                    format!("{:?}", tok.kind),
                    tok.literal
                );
            }
            println!();
            println!("Total tokens: {}", tokens.len());
        }
        Err(e) => {
            eprintln!("LEXER ERROR: {}", e);
            process::exit(1);
        }
    }
}

fn load_program_recursive(source: &str, _base_filename: &str) -> Result<ast::Program, ()> {
    let mut lex = lexer::Lexer::new(source);
    let tokens = match lex.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("LEXER ERROR: {}", e);
            return Err(());
        }
    };

    let mut p = parser::Parser::new(tokens);
    let mut program = match p.parse_program() {
        Ok(prog) => prog,
        Err(e) => {
            eprintln!("PARSE ERROR: {}", e);
            return Err(());
        }
    };

    let mut loaded_paths = HashSet::new();
    let mut pending_uses = Vec::new();

    for item in &program.items {
        if let ast::Item::UseDecl(u) = item {
            pending_uses.push(u.clone());
        }
    }

    let mut added_items = Vec::new();
    let base_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    while let Some(u) = pending_uses.pop() {
        let mut path = base_dir.clone();
        for segment in &u.path {
            path.push(segment);
        }
        path.set_extension("ox");

        let path_str = path.to_string_lossy().into_owned();
        if loaded_paths.contains(&path_str) {
            continue;
        }

        loaded_paths.insert(path_str.clone());

        let source = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading module '{}': {}", path_str, e);
                return Err(());
            }
        };

        let mut sub_lex = lexer::Lexer::new(&source);
        let sub_tokens = match sub_lex.tokenize() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("LEXER ERROR in '{}': {}", path_str, e);
                return Err(());
            }
        };

        let mut sub_p = parser::Parser::new(sub_tokens);
        let sub_program = match sub_p.parse_program() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("PARSE ERROR in '{}': {}", path_str, e);
                return Err(());
            }
        };

        for item in sub_program.items {
            if let ast::Item::UseDecl(sub_u) = &item {
                pending_uses.push(sub_u.clone());
            }
            added_items.push(item);
        }
    }

    program.items.extend(added_items);
    Ok(program)
}

fn run_parser(source: &str, filename: &str) {
    match load_program_recursive(source, filename) {
        Ok(program) => {
            println!("╔══════════════════════════════════════════════════╗");
            println!("║          OXIDE COMPILER — AST OUTPUT            ║");
            println!("╚══════════════════════════════════════════════════╝");
            println!();
            println!("{:#?}", program);
            println!();
            println!("Total top-level items: {}", program.items.len());
        }
        Err(_) => process::exit(1),
    }
}

fn run_checker(source: &str, filename: &str) {
    let program = match load_program_recursive(source, filename) {
        Ok(prog) => prog,
        Err(_) => process::exit(1),
    };

    let mut az = analyzer::Analyzer::new();
    let ok = az.analyze(&program);

    println!("╔══════════════════════════════════════════════════╗");
    println!("║       OXIDE COMPILER — SEMANTIC ANALYSIS        ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();

    if ok {
        println!("  ✓ No semantic errors found.");
    } else {
        for err in &az.errors {
            println!("  ✗ {}", err);
        }
        println!();
        println!("  {} error(s) found.", az.errors.len());
    }

    if !az.warnings.is_empty() {
        println!();
        for w in &az.warnings {
            println!("  ⚠ {}", w);
        }
    }
}

fn run_compile(source: &str, filename: &str) {
    let program = match load_program_recursive(source, filename) {
        Ok(prog) => prog,
        Err(_) => process::exit(1),
    };

    let mut az = analyzer::Analyzer::new();
    let ok = az.analyze(&program);

    if !ok {
        eprintln!("╔══════════════════════════════════════════════════╗");
        eprintln!("║    COMPILE ABORTED — SEMANTIC ERRORS FOUND      ║");
        eprintln!("╚══════════════════════════════════════════════════╝");
        eprintln!();
        for err in &az.errors {
            eprintln!("  ✗ {}", err);
        }
        process::exit(1);
    }

    let mut cg = codegen::CodeGen::new(az.field_access_map);
    cg.generate(&program);

    let mut val = validator::Validator::new();
    val.validate(&cg.instructions);

    if !val.errors.is_empty() {
        eprintln!("╔══════════════════════════════════════════════════╗");
        eprintln!("║       COMPILE ABORTED — OxIR VALIDATION FAILED  ║");
        eprintln!("╚══════════════════════════════════════════════════╝");
        eprintln!();
        for err in &val.errors {
            eprintln!("  ✗ [Inst {}] {:?}: {}", err.instruction_index, err.code, err.message);
        }
        process::exit(1);
    }

    println!("╔══════════════════════════════════════════════════╗");
    println!("║          OXIDE COMPILER — OxIR OUTPUT           ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
    println!("{}", cg.dump());
    println!("  {} OxIR instructions emitted.", cg.instructions.len());
    println!("  ✓ OxIR Validation Pass successful.");
}

fn run_build(source: &str, filename: &str) {
    let program = match load_program_recursive(source, filename) {
        Ok(prog) => prog,
        Err(_) => process::exit(1),
    };

    let mut az = analyzer::Analyzer::new();
    if !az.analyze(&program) {
        eprintln!("SEMANTIC ERRORS FOUND. Build aborted.");
        process::exit(1);
    }

    let mut cg = codegen::CodeGen::new(az.field_access_map);
    cg.generate(&program);

    let mut val = validator::Validator::new();
    val.validate(&cg.instructions);
    if !val.errors.is_empty() {
        eprintln!("OxIR VALIDATION FAILED. Build aborted.");
        for err in &val.errors {
            eprintln!("  ✗ [Inst {}] {:?}: {}", err.instruction_index, err.code, err.message);
        }
        process::exit(1);
    }

    let mut backend = backend::CGenerator::new();
    backend.generate_structs(&program);
    backend.generate(&cg.instructions, &program);

    let out_name = filename.replace(".ox", ".c");
    fs::write(&out_name, backend.output).expect("Failed to write C output file");

    println!("╔══════════════════════════════════════════════════╗");
    println!("║          OXIDE COMPILER — BUILD SUCCESS         ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
    println!("  Source: {}", filename);
    println!("  Output: {}", out_name);
    println!("  ✓ Lexing, Parsing, Semantic Analysis, and OxIR validation passed.");
    println!("  ✓ Transpiled heavily optimized memory-safe OxIR to clean C.");
    println!();
    println!("  To compile to a native executable, run:");
    println!("    gcc -O3 {} -o {}", out_name, filename.replace(".ox", ""));
}
