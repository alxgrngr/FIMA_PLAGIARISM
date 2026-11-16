# ARCHITECTURE

Single Cargo package with native-testable core library and Leptos CSR binary. UI state is volatile signals. Core retains UTF-8 byte ranges into original text and separately normalizes paragraph display to NFC. No server calls process documents. Static assets are deployed to Pages. Future parser and model modules must use bounded work and cancellation before accepting larger inputs.
