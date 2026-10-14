use std::fs;
use std::time::Instant;
use trace_lsp::parser::TraceDocument;

#[test]
fn test_benchmark_v5() {
    let path = "/home/sean/Github/template/test/CCC/test/tx-trace-test.trace";
    if !std::path::Path::new(path).exists() {
        println!("File not found, skipping");
        return;
    }

    let content = fs::read_to_string(path).unwrap();
    println!("File size: {} bytes, {} lines", content.len(), content.lines().count());

    let start = Instant::now();
    let doc = TraceDocument::parse(&content);
    let parse_time = start.elapsed();
    println!("TraceDocument::parse took: {:?}", parse_time);
    println!("Roots count: {}", doc.roots.len());

    let start_sym = Instant::now();
    let symbols = doc.to_document_symbols();
    let sym_time = start_sym.elapsed();
    println!("to_document_symbols took: {:?}", sym_time);

    use std::collections::HashMap;
    fn count_symbols_by_kind(symbols: &[lsp_types::DocumentSymbol], counts: &mut HashMap<String, usize>) {
        for s in symbols {
            *counts.entry(format!("{:?}", s.kind)).or_insert(0) += 1;
            if let Some(children) = &s.children {
                count_symbols_by_kind(children, counts);
            }
        }
    }

    let mut counts = HashMap::new();
    count_symbols_by_kind(&symbols, &mut counts);
    let mut total_symbols = 0;
    for (k, v) in counts {
        println!("Kind {}: {}", k, v);
        total_symbols += v;
    }

    let json = serde_json::to_string(&symbols).unwrap();
    println!("Total DocumentSymbols: {}", total_symbols);
    println!("DocumentSymbols JSON size: {} bytes", json.len());

    let start_tokens = Instant::now();
    let tokens = trace_lsp::tokens::tokenize_document(&content);
    let token_time = start_tokens.elapsed();
    println!("tokenize_document took: {:?}", token_time);
    println!("Total semantic tokens: {}", tokens.data.len());
}


