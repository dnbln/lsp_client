# lsp_client

## How to use

You can follow the example in the main.rs on how to use the library, the benefit of this api is not having to deal with the json rpc protocol, you can just call the methods and get the results.

To run the example in main.rs:

1. Install https://github.com/typescript-language-server/typescript-language-server
2. Install the `lsp_client` binary using `cargo install --path .`
3. In a directory containing TypeScript files, run `lsp_client ./path/to/code.ts`

## Why did we build this?

LSP have a special JSON RPC protocol.  It is not straightforward to use, and you probably want to make sure that the requests are handled correctly, so we are share our implementation in the hopes that others can use it for fun and profit.
