(process_definition
  "process" @context
  name: (identifier) @name) @item

(workflow_definition
  "workflow" @context
  name: (identifier) @name) @item

(workflow_definition
  "workflow" @name
  !name) @item

(function_definition
  "def"? @context
  name: (identifier) @name) @item

(record_definition
  "record" @context
  name: (identifier) @name) @item

(enum_definition
  "enum" @context
  name: (identifier) @name) @item
