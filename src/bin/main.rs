use client::start_language_server;

use lsp_client::lsp::client;
use lsp_types::GotoDefinitionParams;
use lsp_types::Position;
use lsp_types::TextDocumentIdentifier;
use lsp_types::TextDocumentPositionParams;
use serde_json::json;
use std::process::Stdio;
use tokio::process::Child;
use tokio::process::Command;
use tokio::sync::oneshot;

use lsp_types::ClientCapabilities;
use lsp_types::CodeActionClientCapabilities;
use lsp_types::CodeLensClientCapabilities;
use lsp_types::DynamicRegistrationClientCapabilities;
use lsp_types::ExecuteCommandClientCapabilities;
use lsp_types::GotoCapability;
use lsp_types::InitializeParams;
use lsp_types::RenameClientCapabilities;
use lsp_types::SignatureHelpClientCapabilities;
use lsp_types::TextDocumentClientCapabilities;
use lsp_types::WorkDoneProgressParams;
use lsp_types::WorkspaceClientCapabilities;
use lsp_types::WorkspaceFolder;
use url::Url;

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    let file_name = std::path::PathBuf::from(args.nth(1).expect("No filename argument supplied"));
    let absolute_file_name = file_name.canonicalize().unwrap_or_else(|err| {
        panic!(
            "Failed getting absolute path for {}: {err}",
            file_name.display()
        )
    });
    let file_name_url = format!("file://{}", absolute_file_name.display());

    println!("starting main read loop");
    let (_child, lang_server) = start_language_server(prepare_command()).await;

    let working_directory = std::env::current_dir().expect("Unable to get current directory");
    eprintln!("cwd: {}", working_directory.display());

    if !absolute_file_name.starts_with(working_directory.clone()) {
        eprintln!(
            "{} must reside under the current working directory.",
            absolute_file_name.display()
        );
        std::process::exit(1);
    }

    let working_directory_url = format!("file://{}", working_directory.display());
    let working_directory_uri = Url::parse(&working_directory_url).unwrap_or_else(|err| {
        panic!(
            "Failed converting directory name {} into a Url: {err}",
            working_directory.display()
        )
    });

    // Prepare the initialize request
    let init_params = InitializeParams {
        process_id: None, // Super important to set it to NONE https://github.com/typescript-language-server/typescript-language-server/issues/262
        root_uri: Some(working_directory_uri.clone()),
        initialization_options: Some(serde_json::json!({
            "hostInfo": "vscode",
            "maxTsServerMemory": 4096 * 2,
            "tsserver": {
                "logDirectory": "/tmp/tsserver",
                "logVerbosity": "verbose",
                "maxTsServerMemory": 4096 * 2,
            },
            "preferences": {
                "providePrefixAndSuffixTextForRename": true,
                "allowRenameOfImportPath": true,
                "includePackageJsonAutoImports": "auto",
                "excludeLibrarySymbolsInNavTo": true
            }
        })),
        capabilities: ClientCapabilities {
            text_document: Some(TextDocumentClientCapabilities {
                declaration: Some(GotoCapability {
                    dynamic_registration: Some(true),
                    link_support: Some(true),
                }),
                definition: Some(GotoCapability {
                    dynamic_registration: Some(true),
                    link_support: Some(true),
                }),
                code_action: Some(CodeActionClientCapabilities {
                    dynamic_registration: Some(true),
                    ..Default::default()
                }),
                code_lens: Some(CodeLensClientCapabilities {
                    dynamic_registration: Some(true),
                }),
                implementation: Some(GotoCapability {
                    dynamic_registration: Some(true),
                    link_support: Some(true),
                }),
                references: Some(DynamicRegistrationClientCapabilities {
                    dynamic_registration: Some(true),
                }),
                rename: Some(RenameClientCapabilities {
                    dynamic_registration: Some(true),
                    ..Default::default()
                }),
                signature_help: Some(SignatureHelpClientCapabilities {
                    dynamic_registration: Some(true),
                    ..Default::default()
                }),
                synchronization: Some(lsp_types::TextDocumentSyncClientCapabilities {
                    dynamic_registration: Some(true),
                    will_save: Some(true),
                    will_save_wait_until: Some(true),
                    did_save: Some(true),
                }),
                ..Default::default()
            }),
            workspace: Some(WorkspaceClientCapabilities {
                execute_command: Some(ExecuteCommandClientCapabilities {
                    dynamic_registration: Some(true),
                }),
                did_change_configuration: Some(
                    lsp_types::DidChangeConfigurationClientCapabilities {
                        dynamic_registration: Some(true),
                    },
                ),
                did_change_watched_files: Some(
                    lsp_types::DidChangeWatchedFilesClientCapabilities {
                        dynamic_registration: Some(true),
                        relative_pattern_support: Some(true),
                    },
                ),
                symbol: Some(lsp_types::WorkspaceSymbolClientCapabilities {
                    dynamic_registration: Some(true),
                    symbol_kind: Some(lsp_types::SymbolKindCapability {
                        value_set: Some(vec![
                            lsp_types::SymbolKind::FILE,
                            lsp_types::SymbolKind::MODULE,
                            lsp_types::SymbolKind::NAMESPACE,
                            lsp_types::SymbolKind::PACKAGE,
                            lsp_types::SymbolKind::CLASS,
                            lsp_types::SymbolKind::METHOD,
                            lsp_types::SymbolKind::PROPERTY,
                            lsp_types::SymbolKind::FIELD,
                            lsp_types::SymbolKind::CONSTRUCTOR,
                            lsp_types::SymbolKind::ENUM,
                            lsp_types::SymbolKind::INTERFACE,
                            lsp_types::SymbolKind::FUNCTION,
                            lsp_types::SymbolKind::VARIABLE,
                            lsp_types::SymbolKind::CONSTANT,
                            lsp_types::SymbolKind::STRING,
                            lsp_types::SymbolKind::NUMBER,
                            lsp_types::SymbolKind::BOOLEAN,
                            lsp_types::SymbolKind::ARRAY,
                            lsp_types::SymbolKind::OBJECT,
                            lsp_types::SymbolKind::KEY,
                            lsp_types::SymbolKind::NULL,
                            lsp_types::SymbolKind::STRUCT,
                            lsp_types::SymbolKind::EVENT,
                            lsp_types::SymbolKind::OPERATOR,
                        ]),
                    }),
                    resolve_support: Some(lsp_types::WorkspaceSymbolResolveSupportCapability {
                        properties: vec!["location.range".to_string()],
                    }),
                    ..Default::default()
                }),
                workspace_edit: Some(lsp_types::WorkspaceEditClientCapabilities {
                    document_changes: Some(true),
                    ..Default::default()
                }),
                workspace_folders: Some(true),
                configuration: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        },
        trace: Some(lsp_types::TraceValue::Verbose),
        client_info: None,
        locale: None,
        workspace_folders: Some(vec![WorkspaceFolder {
            uri: working_directory_uri,
            name: "ide".to_string(),
        }]),
        work_done_progress_params: WorkDoneProgressParams {
            work_done_token: Some(lsp_types::NumberOrString::Number(2)),
        },
        ..Default::default()
    };

    let (tx, rx) = oneshot::channel();
    lang_server
        .send_request("initialize", &json!(init_params), |result| {
            println!("received response {:?}", result);
            tx.send(result).expect("unable to send to receiver");
        })
        .await;
    let result = rx.await;
    dbg!(&result);

    // Now we send over the open text document notification
    let file_name_uri = Url::parse(&file_name_url).unwrap_or_else(|err| {
        panic!("Failed converting file name {file_name_url} into a Url: {err}")
    });
    let file_contents = std::fs::read_to_string(file_name.clone())
        .unwrap_or_else(|err| panic!("Failed to read file {}: {err}", file_name.display()));

    lang_server
        .send_notification(
            "textDocument/didOpen",
            &json!({
                "textDocument": {
                    "uri": file_name_url,
                    "languageId": "typescript",
                    "version": 1,
                    "text": file_contents
                }
            }),
        )
        .await;

    // now we ask for a goto definition
    let position = Position {
        line: 6,
        character: 14,
    };
    let go_to_definition_request = GotoDefinitionParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: file_name_uri },
            position,
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };
    let (tx, rx) = oneshot::channel();
    lang_server
        .send_request(
            "textDocument/typeDefinition",
            &json!(go_to_definition_request),
            |result| {
                println!("received response goto definition {:?}", result);
                tx.send(result).expect("unable to send to receiver");
            },
        )
        .await;
    dbg!(&rx.await);
}

fn prepare_command() -> Child {
    // Start the TypeScript language server
    let child = Command::new("typescript-language-server")
        .args(&["--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("NODE_OPTIONS", "--max-old-space-size=3072")
        .spawn()
        .expect("Failed to start typescript-language-server");
    let process_id = child.id();
    println!("{:?}", process_id);
    child
}
