;; Ported from https://github.com/nextflow-io/tree-sitter-nextflow/blob/v0.4.0/queries/highlights.scm
;; with capture names mapped onto Zed theme keys. Later patterns take precedence.

(identifier) @variable

((identifier) @variable.special
  (#any-of? @variable.special
    "params" "task" "workflow" "nextflow" "log" "launchDir" "moduleDir"
    "projectDir" "workDir" "baseDir" "secrets"))

((identifier) @type.builtin
  (#any-of? @type.builtin "Channel" "channel"))

;; ========================================
;; KEYWORDS
;; ========================================

[
  "process"
  "workflow"
  "agent"
  "include"
  "from"
  "import"
  "nextflow"
  "params"
  "record"
  "enum"
  "tuple"
  "def"
  "as"
  "new"
] @keyword

[
  "if"
  "else"
  "for"
  "try"
  "catch"
  "finally"
  "return"
  "throw"
  "assert"
] @keyword.control

[
  "in"
  "!in"
  "instanceof"
  "!instanceof"
] @keyword.operator

;; Section labels
[
  "input"
  "output"
  "stage"
  "topic"
  "when"
  "script"
  "shell"
  "exec"
  "stub"
  "prompt"
  "take"
  "main"
  "emit"
  "publish"
  "onComplete"
  "onError"
] @label

(output_definition "output" @keyword)

(labeled_statement label: (identifier) @label)

;; ========================================
;; DEFINITIONS
;; ========================================

(process_definition name: (identifier) @function)
(workflow_definition name: (identifier) @function)
(agent_definition name: (identifier) @function)
(function_definition name: (identifier) @function)
(include_item name: (identifier) @function)
(include_item alias: (identifier) @function)

(parameter name: (identifier) @variable.parameter)
(workflow_take name: (identifier) @variable.parameter)
(process_input name: (identifier) @variable.parameter)

(param_declaration name: (identifier) @property)
(param_assignment (identifier) @property)
(feature_flag (identifier) @property)
(record_field name: (identifier) @property)
(workflow_emit name: (identifier) @property)
(workflow_publish name: (identifier) @property)
(process_output name: (identifier) @property)
(output_declaration name: (identifier) @property)

(record_definition name: (identifier) @type)
(enum_definition name: (identifier) @type)
(enum_constant) @constant
(type (identifier) @type)

;; ========================================
;; CALLS AND PROPERTIES
;; ========================================

(member_expression property: (identifier) @property)
(named_argument name: (identifier) @property)

(call_expression function: (identifier) @function.call)
(call_expression function: (member_expression property: (identifier) @function.method))
(command_expression function: (identifier) @function.call)
(command_expression function: (member_expression property: (identifier) @function.method))

;; Process directives: tag "x", cpus 4, memory { 2.GB * task.attempt }
(process_definition
  (expression_statement
    [
      (command_expression function: (identifier) @attribute)
      (call_expression function: (identifier) @attribute)
    ]))

;; Legacy input/output qualifiers: val x, path "*.bam", tuple val(meta), path(x)
(input_section
  (expression_statement
    (command_expression function: (identifier) @type.builtin)))
(output_section
  (expression_statement
    (command_expression function: (identifier) @type.builtin)))
(input_section
  (expression_statement
    (command_expression
      arguments: (argument_list (call_expression function: (identifier) @type.builtin)))))
(output_section
  (expression_statement
    (command_expression
      arguments: (argument_list (call_expression function: (identifier) @type.builtin)))))
((output_section (process_output name: (identifier) @type.builtin))
  (#any-of? @type.builtin "stdout" "stdin"))

;; ========================================
;; OPERATORS AND PUNCTUATION
;; ========================================

[
  "="
  "+="
  "-="
  "*="
  "/="
  "%="
  "**="
  "<<="
  ">>="
  ">>>="
  "&="
  "|="
  "^="
  "?="
] @operator.assignment

[
  "=="
  "!="
  "<"
  ">"
  "<="
  ">="
  "&&"
  "||"
  "=~"
  "==~"
  "<=>"
  "?:"
  "?"
  "!"
  "+"
  "-"
  "*"
  "/"
  "%"
  "**"
  ".."
  "..<"
  "<<"
  ">>"
  ">>>"
  "&"
  "^"
  "~"
  "?."
  "*."
] @operator

;; Channel pipes and closure arrows
[
  "|"
  "->"
] @operator.channel

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

[
  ","
  ":"
  "."
] @punctuation.delimiter

;; ========================================
;; LITERALS
;; ========================================

(string) @string
(string_content) @string
(escape_sequence) @string.escape
(interpolation ["${" "$" "}"] @punctuation.special)
(slashy_string) @string.regex

(integer_literal) @number
(float_literal) @number.float
(boolean_literal) @boolean
(null_literal) @constant.builtin

;; ========================================
;; COMMENTS
;; ========================================

(line_comment) @comment
(block_comment) @comment
(shebang) @preproc
