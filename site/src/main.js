import './style.css'

document.querySelector('#app').innerHTML = `
    <!-- Navigation -->
    <nav>
        <div class="logo">⚡ ReluxScript</div>
        <ul class="nav-links">
            <li><a href="#features">Features</a></li>
            <li><a href="https://docs.reluxscript.com">Docs</a></li>
            <li><a href="#examples">Examples</a></li>
            <li><a href="https://github.com/reluxscript/reluxscript">GitHub</a></li>
        </ul>
    </nav>

    <!-- Hero Section -->
    <section class="hero">
        <img src="/lux-image-4.png" alt="ReluxScript Logo" class="hero-logo">

        <h1>ReluxScript</h1>

        <div class="tagline">
            <span>⚡ Light,</span>
            <span>Light,</span>
            <span>Write!</span>
        </div>

        <p class="subtitle">
            Write AST transformations once. Compile to Babel, SWC, and beyond.<br>
            One <code>.lux</code> file. Infinite possibilities.
        </p>

        <div class="cta-buttons">
            <a href="#" class="btn btn-primary">Get Started →</a>
            <a href="#" class="btn btn-secondary">View Examples</a>
        </div>
    </section>

    <!-- Features Section -->
    <section class="features" id="features">
        <h2 class="section-title">Why ReluxScript?</h2>

        <div class="feature-grid">
            <div class="feature-card">
                <div class="feature-icon">🔺</div>
                <h3>Write Once</h3>
                <p>Stop maintaining duplicate Babel and SWC plugins. Write your AST transformation logic once in clean, Rust-inspired syntax.</p>
            </div>

            <div class="feature-card">
                <div class="feature-icon">⚡</div>
                <h3>Dual Compilation</h3>
                <p>Compile to both Babel (JavaScript) and SWC (Rust/WASM) from a single source. Perfect for modern toolchains.</p>
            </div>

            <div class="feature-card">
                <div class="feature-icon">✨</div>
                <h3>Type Safety</h3>
                <p>Catch errors at compile time with static type checking. Your transformations are validated before they run.</p>
            </div>

            <div class="feature-card">
                <div class="feature-icon">🎯</div>
                <h3>Vector Alignment</h3>
                <p>Uses the intersection of Babel and SWC features, not the union. What compiles will work correctly on both platforms.</p>
            </div>

            <div class="feature-card">
                <div class="feature-icon">🔧</div>
                <h3>Familiar Syntax</h3>
                <p>Rust-inspired syntax that feels natural to systems programmers while being accessible to JavaScript developers.</p>
            </div>

            <div class="feature-card">
                <div class="feature-icon">🚀</div>
                <h3>Extensible</h3>
                <p>Beyond Babel and SWC—compile to custom transpilers like TSX→C# for innovative architectures.</p>
            </div>
        </div>
    </section>

    <!-- Code Example Section -->
    <section class="code-example" id="examples">
        <div class="code-container">
            <h2 class="section-title">See It In Action</h2>

            <div class="code-block">
                <div class="code-label">.lux source</div>
                <pre><code><span class="comment">// Write once in ReluxScript</span>
<span class="keyword">plugin</span> <span class="function">RemoveConsole</span> {
    <span class="keyword">fn</span> <span class="function">visit_call_expression</span>(node: &<span class="keyword">mut</span> CallExpression, ctx: &Context) {
        <span class="keyword">if</span> <span class="function">matches!</span>(node.callee, <span class="string">"console.log"</span>) {
            *node = Statement::<span class="function">empty</span>();
        }
    }
}</code></pre>
            </div>

            <div class="arrow-down">⬇️ Compiles to ⬇️</div>

            <div class="output-grid">
                <div class="code-block">
                    <div class="code-label">Babel (JavaScript)</div>
                    <pre><code><span class="keyword">module</span>.<span class="keyword">exports</span> = <span class="keyword">function</span>({ types: t }) {
  <span class="keyword">return</span> {
    visitor: {
      <span class="function">CallExpression</span>(path) {
        <span class="keyword">if</span> (
          t.<span class="function">isMemberExpression</span>(path.node.callee) &&
          path.node.callee.object.name === <span class="string">'console'</span> &&
          path.node.callee.property.name === <span class="string">'log'</span>
        ) {
          path.<span class="function">remove</span>();
        }
      }
    }
  };
};</code></pre>
                </div>

                <div class="code-block">
                    <div class="code-label">SWC (Rust)</div>
                    <pre><code><span class="keyword">pub struct</span> <span class="function">RemoveConsole</span>;

<span class="keyword">impl</span> VisitMut <span class="keyword">for</span> RemoveConsole {
    <span class="keyword">fn</span> <span class="function">visit_mut_call_expr</span>(
        &<span class="keyword">mut self</span>,
        node: &<span class="keyword">mut</span> CallExpr
    ) {
        <span class="keyword">if</span> <span class="function">is_console_log</span>(node) {
            *node = <span class="function">empty_stmt</span>();
        }
    }
}</code></pre>
                </div>
            </div>
        </div>
    </section>

    <!-- Footer -->
    <footer>
        <div class="footer-links">
            <a href="https://docs.reluxscript.com">Documentation</a>
            <a href="#examples">Examples</a>
            <a href="https://github.com/reluxscript/reluxscript">GitHub</a>
            <a href="#community">Community</a>
            <a href="#blog">Blog</a>
        </div>
        <p class="copyright">
            ReluxScript © 2025 • Light, Light, Write! ⚡
        </p>
    </footer>
`
