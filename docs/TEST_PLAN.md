# TEST PLAN

Native tests cover whitespace counts, paragraph mapping, CRLF and Unicode, NFC and length/empty errors. Required next: TXT browser success/error/race cases, XSS text rendering, responsive keyboard navigation, DOCX ZIP bomb and PDF resource-limit tests when implemented, classifier threshold and metadata tests once real model exists. Chrome/Edge/Firefox/Safari and GPU checks require actual browser execution. Never count nonexistent inference as CPU fallback validation.
