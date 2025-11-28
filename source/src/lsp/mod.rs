//! Language Server Protocol implementation for ReluxScript

mod server;
mod handlers;
mod diagnostics;
mod completions;

pub use server::ReluxScriptLanguageServer;

use tower_lsp::LspService;
use tower_lsp::Server;

pub async fn start_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| {
        ReluxScriptLanguageServer::new(client)
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
