;; Bash injection into process script/shell/stub bodies: the string that ends
;; the section. Only the content chunks are captured, so quote delimiters and
;; ${...} interpolations stay Nextflow. injection.combined merges the chunks
;; around interpolations into one bash document.
;;
;; exec: bodies are Groovy, so they are left alone.

(script_section
  ["script" "shell"]
  (expression_statement
    (string (string_content) @injection.content)) .
  (#set! injection.language "bash")
  (#set! injection.combined))

(stub_section
  (expression_statement
    (string (string_content) @injection.content)) .
  (#set! injection.language "bash")
  (#set! injection.combined))

;; A process with no section labels: the body is an implicit script.
(process_definition
  (expression_statement
    (string (string_content) @injection.content)) .
  (#set! injection.language "bash")
  (#set! injection.combined))
