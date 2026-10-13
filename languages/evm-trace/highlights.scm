; ==============================================================================
; Tree-sitter Highlights Query for EVM Execution Traces (evm_trace)
; Production-grade syntax highlighting for Zed editor
; ==============================================================================

; --- Comments ---
(comment) @comment

; --- Tree Guides & Graph Structure ---
(tree_guide) @punctuation.delimiter

; --- Operators & Arrows ---
"::" @operator
"@" @punctuation.delimiter
(creation_arrow) @operator
(return_arrow) @operator
(change_arrow) @operator

; --- Brackets & Delimiters ---
"[" @punctuation.bracket
"]" @punctuation.bracket
"{" @punctuation.bracket
"}" @punctuation.bracket
"(" @punctuation.bracket
")" @punctuation.bracket
":" @punctuation.delimiter
"," @punctuation.delimiter

; --- Keywords & Control ---
(keyword) @keyword
(header_keyword) @keyword
(call_type) @keyword
(storage_keyword) @keyword
(status_keyword) @type
(revert_keyword) @keyword
(bytes_keyword) @keyword

; --- Calls, Functions & Events ---
(call_expression
  target: (identifier) @type)

(call_expression
  target: (hex_literal) @constant.numeric)

(call_expression
  function: (identifier) @function.method)

(call_expression
  function: (hex_selector) @function.method)

(emit_trace
  name: (identifier) @type)

(custom_error
  name: (identifier) @function.method)

; --- Contract Creation ---
(creation_trace
  contract: (identifier) @type)

(creation_trace
  contract: (template_value) @type)

(creation_trace
  address: (hex_literal) @constant.numeric)

; --- Gas & Estimations ---
(gas
  amount: (number) @constant.numeric)

(estimate_value) @constant.numeric

; --- Properties, Labels & Slots ---
(named_argument
  name: (identifier) @property)

(labeled_address
  label: (identifier) @property
  address: (hex_literal) @constant.numeric)

(storage_change
  slot: (identifier) @property)

(storage_change
  slot: (number) @constant.numeric)

(storage_change
  slot: (hex_literal) @constant.numeric)

(storage_change
  before: (number) @constant.numeric)

(storage_change
  before: (hex_literal) @constant.numeric)

(storage_change
  after: (number) @constant.numeric)

(storage_change
  after: (hex_literal) @constant.numeric)

(topic_entry
  index: (number) @constant.numeric
  value: (hex_literal) @constant.numeric)

(data_entry
  value: (hex_literal) @constant.numeric)

; --- Values & Literals ---
(number) @constant.numeric
(hex_literal) @constant.numeric
(raw_hex) @constant.numeric
(string) @string
(template_value) @string
(boolean) @constant.numeric
(revert_message) @string

; Fallback identifiers
(identifier) @variable
