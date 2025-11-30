import * as path from 'path';
import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    Executable
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    console.log('ReluxScript extension activating...');

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('reluxscript.compile.babel', compileToBabel)
    );
    context.subscriptions.push(
        vscode.commands.registerCommand('reluxscript.compile.swc', compileToSwc)
    );
    context.subscriptions.push(
        vscode.commands.registerCommand('reluxscript.compile.both', compileToBoth)
    );

    // Start language server
    startLanguageServer(context);

    console.log('ReluxScript extension activated');
}

function startLanguageServer(context: vscode.ExtensionContext) {
    // Find the language server executable
    const serverPath = findLanguageServer();

    if (!serverPath) {
        vscode.window.showWarningMessage(
            'ReluxScript language server not found. Install ReluxScript or build with: cargo build --features lsp --bin reluxscript-lsp'
        );
        return;
    }

    console.log(`Found ReluxScript LSP at: ${serverPath}`);

    // Define how to start the server
    const run: Executable = {
        command: serverPath,
        options: {
            env: process.env
        }
    };

    const serverOptions: ServerOptions = {
        run,
        debug: run
    };

    // Options for the language client
    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'reluxscript' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.lux')
        }
    };

    // Create and start the client
    client = new LanguageClient(
        'reluxscriptLanguageServer',
        'ReluxScript Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
}

function findLanguageServer(): string | null {
    const fs = require('fs');

    // Look for reluxscript-lsp in several locations:

    // 1. Development build (relative to extension)
    const devPath = path.join(__dirname, '../../source/target/debug/reluxscript-lsp');
    if (fs.existsSync(devPath)) {
        return devPath;
    }

    const devPathExe = devPath + '.exe';
    if (fs.existsSync(devPathExe)) {
        return devPathExe;
    }

    // 2. Release build (relative to extension)
    const releasePath = path.join(__dirname, '../../source/target/release/reluxscript-lsp');
    if (fs.existsSync(releasePath)) {
        return releasePath;
    }

    const releasePathExe = releasePath + '.exe';
    if (fs.existsSync(releasePathExe)) {
        return releasePathExe;
    }

    // 3. Bundled binary
    const bundledPath = path.join(__dirname, '../bin/reluxscript-lsp');
    if (fs.existsSync(bundledPath)) {
        return bundledPath;
    }

    const bundledPathExe = bundledPath + '.exe';
    if (fs.existsSync(bundledPathExe)) {
        return bundledPathExe;
    }

    // 4. System PATH
    // TODO: Try to find in PATH using 'which' or 'where'

    return null;
}

async function compileToBabel() {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== 'reluxscript') {
        vscode.window.showWarningMessage('No ReluxScript file is active');
        return;
    }

    const document = editor.document;
    await document.save();

    const terminal = vscode.window.createTerminal('ReluxScript');
    terminal.show();
    terminal.sendText(`relux build "${document.fileName}" --target babel`);
}

async function compileToSwc() {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== 'reluxscript') {
        vscode.window.showWarningMessage('No ReluxScript file is active');
        return;
    }

    const document = editor.document;
    await document.save();

    const terminal = vscode.window.createTerminal('ReluxScript');
    terminal.show();
    terminal.sendText(`relux build "${document.fileName}" --target swc`);
}

async function compileToBoth() {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== 'reluxscript') {
        vscode.window.showWarningMessage('No ReluxScript file is active');
        return;
    }

    const document = editor.document;
    await document.save();

    const terminal = vscode.window.createTerminal('ReluxScript');
    terminal.show();
    terminal.sendText(`relux build "${document.fileName}"`);
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
