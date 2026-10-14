use lsp_types::{
    DocumentHighlight, DocumentSymbol, FoldingRange, FoldingRangeKind, Hover,
    HoverContents, MarkupContent, MarkupKind, Position, Range, SymbolKind,
};
use crate::parser::{CallType, TraceDocument, TraceItemKind, TraceNode};

pub fn format_gas(gas: u64) -> String {
    if gas >= 1_000_000 {
        format!("{:.2}M", gas as f64 / 1_000_000.0)
    } else if gas >= 1_000 {
        format!("{:.1}k", gas as f64 / 1_000.0)
    } else {
        format!("{gas}")
    }
}

fn truncate_addr(addr: &str) -> String {
    if addr.starts_with("0x") && addr.len() >= 10 {
        format!("{}...{}", &addr[..6], &addr[addr.len() - 4..])
    } else {
        addr.to_string()
    }
}

fn truncate_val(val: &str) -> String {
    let trimmed = val.trim();
    if trimmed.len() > 24 {
        format!("{}...", &trimmed[..24])
    } else {
        trimmed.to_string()
    }
}

pub fn node_to_document_symbol(node: &TraceNode) -> Option<DocumentSymbol> {
    use crate::parser::ReturnStatus;

    let (name, detail, kind) = match &node.kind {
        TraceItemKind::RootCall { gas, target, function } => {
            let ret = match &node.return_status {
                Some(ReturnStatus::Success(v)) if !v.is_empty() => format!(" -> [Return] {}", truncate_val(v)),
                Some(ReturnStatus::Stop) => " -> [Stop]".to_string(),
                Some(ReturnStatus::Revert(r)) => format!(" -> [Revert] {}", truncate_val(r)),
                _ => String::new(),
            };
            (
                format!("{target}::{function}() [{}]", format_gas(*gas)),
                Some(format!("Root Call{ret}")),
                SymbolKind::METHOD,
            )
        }
        TraceItemKind::Call {
            gas,
            target,
            function,
            call_type,
            value,
            ..
        } => {
            let type_tag = match call_type {
                CallType::StaticCall => " [staticcall]",
                CallType::DelegateCall => " [delegatecall]",
                CallType::Create => " [create]",
                CallType::Call => "",
            };
            let val_tag = if let Some(v) = value {
                format!(" {{val: {}}}", truncate_addr(v))
            } else {
                String::new()
            };
            let ret = match &node.return_status {
                Some(ReturnStatus::Success(v)) if !v.is_empty() => format!(" -> [Return] {}", truncate_val(v)),
                Some(ReturnStatus::Stop) => " -> [Stop]".to_string(),
                Some(ReturnStatus::Revert(r)) => format!(" -> [Revert] {}", truncate_val(r)),
                _ => String::new(),
            };
            let short_target = truncate_addr(target);
            (
                format!("{short_target}::{function} [{}{val_tag}]", format_gas(*gas)),
                Some(format!("call{type_tag}{ret}")),
                SymbolKind::FUNCTION,
            )
        }
        TraceItemKind::Create { gas, contract, address } => (
            format!("new {}@{} [{}]", contract, truncate_addr(address), format_gas(*gas)),
            Some("deploy".to_string()),
            SymbolKind::CONSTRUCTOR,
        ),
        TraceItemKind::Event { name, details } => {
            let short_details = if details.len() > 40 {
                format!("{}...", &details[..40])
            } else {
                details.clone()
            };
            (
                format!("emit {name}{short_details}"),
                Some("event".to_string()),
                SymbolKind::EVENT,
            )
        }
        TraceItemKind::StorageChanges { .. } => {
            let slot_count = node
                .children
                .iter()
                .filter(|c| matches!(c.kind, TraceItemKind::Other { ref text } if text.starts_with('@')))
                .count();
            let count_tag = if slot_count > 0 {
                format!("{slot_count} slot{}", if slot_count == 1 { "" } else { "s" })
            } else {
                "storage".to_string()
            };
            (
                format!("storage changes ({count_tag})"),
                Some("storage".to_string()),
                SymbolKind::STRUCT,
            )
        }
        TraceItemKind::Other { text } => {
            if text.starts_with('@') {
                (
                    text.clone(),
                    Some("storage slot".to_string()),
                    SymbolKind::PROPERTY,
                )
            } else {
                return None;
            }
        }
        TraceItemKind::Return { .. } => return None,
    };

    let range = Range {
        start: Position {
            line: node.line_start,
            character: 0,
        },
        end: Position {
            line: node.line_end,
            character: node.raw_line.len() as u32,
        },
    };

    let selection_range = Range {
        start: Position {
            line: node.line_start,
            character: 0,
        },
        end: Position {
            line: node.line_start,
            character: node.raw_line.len() as u32,
        },
    };

    let children: Vec<DocumentSymbol> = node
        .children
        .iter()
        .filter_map(node_to_document_symbol)
        .collect();

    #[allow(deprecated)]
    Some(DocumentSymbol {
        name,
        detail,
        kind,
        tags: None,
        deprecated: None,
        range,
        selection_range,
        children: if children.is_empty() { None } else { Some(children) },
    })
}



impl TraceDocument {
    pub fn to_document_symbols(&self) -> Vec<DocumentSymbol> {
        self.roots.iter().filter_map(node_to_document_symbol).collect()
    }

    pub fn to_folding_ranges(&self) -> Vec<FoldingRange> {
        let mut ranges = Vec::new();
        fn collect_ranges(node: &TraceNode, out: &mut Vec<FoldingRange>) {
            if node.line_end > node.line_start {
                out.push(FoldingRange {
                    start_line: node.line_start,
                    start_character: None,
                    end_line: node.line_end,
                    end_character: None,
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: None,
                });
            }
            for child in &node.children {
                collect_ranges(child, out);
            }
        }

        for root in &self.roots {
            collect_ranges(root, &mut ranges);
        }
        ranges
    }

    pub fn hover_at(&self, position: Position) -> Option<Hover> {
        let line_idx = position.line;
        // Find line content
        fn find_node<'a>(nodes: &'a [TraceNode], line: u32) -> Option<&'a TraceNode> {
            for node in nodes {
                if node.line_start == line {
                    return Some(node);
                }
                if line >= node.line_start && line <= node.line_end {
                    if let Some(child_match) = find_node(&node.children, line) {
                        return Some(child_match);
                    }
                    return Some(node);
                }
            }
            None
        }

        let node = find_node(&self.roots, line_idx)?;
        let mut md = String::new();

        match &node.kind {
            TraceItemKind::Call {
                gas,
                target,
                function,
                args,
                call_type,
                value,
            } => {
                let call_type_str = match call_type {
                    CallType::Call => "CALL",
                    CallType::StaticCall => "STATICCALL",
                    CallType::DelegateCall => "DELEGATECALL",
                    CallType::Create => "CREATE",
                };
                md.push_str(&format!("### **{call_type_str}**: `{target}::{function}`\n\n"));
                md.push_str(&format!("- **Gas Consumed**: `{} gas` (`{}`)\n", gas, format_gas(*gas)));

                if let Some(known) = self.addresses.get(target) {
                    md.push_str(&format!("- **Known Label**: `{known}`\n"));
                }
                if let Some(val) = value {
                    md.push_str(&format!("- **Value Attached**: `{val}` wei\n"));
                }
                if !args.is_empty() && args != "()" {
                    md.push_str(&format!("- **Arguments**: `{args}`\n"));
                }
                if let Some(ref status) = node.return_status {
                    md.push_str(&format!("- **Result**: `{:?}`\n", status));
                }
            }
            TraceItemKind::Create { gas, contract, address } => {
                md.push_str(&format!("### **CONTRACT CREATION**: `{contract}`\n\n"));
                md.push_str(&format!("- **Deployed Address**: `{address}`\n"));
                md.push_str(&format!("- **Gas Consumed**: `{} gas` (`{}`)\n", gas, format_gas(*gas)));
            }
            TraceItemKind::Event { name, details } => {
                md.push_str(&format!("### **EVENT EMITTED**: `{name}`\n\n"));
                md.push_str(&format!("- **Parameters**: `{details}`\n"));
            }
            TraceItemKind::StorageChanges { .. } => {
                md.push_str("### **STORAGE CHANGES**\n\nShows contract state modifications made in this call scope.\n");
            }
            _ => {
                md.push_str(&format!("```\n{}\n```", node.raw_line.trim()));
            }
        }

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: md,
            }),
            range: None,
        })
    }

    pub fn highlights_at(&self, _position: Position) -> Option<Vec<DocumentHighlight>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_symbols_and_folding() {
        let content = include_str!("../../../sample.trace");
        let doc = TraceDocument::parse(content);

        // Document symbols
        let symbols = doc.to_document_symbols();
        assert_eq!(symbols.len(), 2, "Expected 2 root symbols");
        assert!(symbols[0].name.contains("CCCTest::setUp"));
        assert!(symbols[1].name.contains("CCCTest::test_profit_success"));

        // Children under test_profit_success
        let test_children = symbols[1].children.as_ref().expect("test_profit_success must have children");
        assert!(!test_children.is_empty(), "test_profit_success should have multiple children");

        // Look for Proof::run() in children
        let proof_run = test_children.iter().find(|s| s.name.contains("Proof::run"));
        assert!(proof_run.is_some(), "Proof::run should exist in symbol tree");
        let proof_children = proof_run.unwrap().children.as_ref().expect("Proof::run should have nested sub-calls");
        assert!(!proof_children.is_empty(), "Proof::run children must not be empty");

        // Folding ranges
        let folding = doc.to_folding_ranges();
        assert!(!folding.is_empty(), "Folding ranges must be generated for nested calls");
        for range in &folding {
            assert!(range.end_line > range.start_line, "Folding range must span across lines");
        }

        // Hover test on root
        let hover = doc.hover_at(Position { line: 12, character: 4 });
        assert!(hover.is_some(), "Hover should return documentation for line 12");
    }
}

