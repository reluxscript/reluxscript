using Minimact.AspNetCore.Core;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace Minimact.Components;

// Example 1: Basic Counter
[Component]
public partial class Counter : MinimactComponent
{
    [State]
    private int count = 0;

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("div", new Dictionary<string, string>(), new VNode[]
        {
            new VElement("h1", $"Count: {count}"),
            new VElement("button", new Dictionary<string, string> { ["onclick"] = "Handle0" }, "Increment"),
            new VElement("button", new Dictionary<string, string> { ["onclick"] = "Handle1" }, "Decrement")
        });
    }

    private void Handle0()
    {
        count = count + 1;
        SetState(nameof(count), count);
    }

    private void Handle1()
    {
        count = count - 1;
        SetState(nameof(count), count);
    }
}

// Example 2: Conditional Rendering
[Component]
public partial class UserProfile : MinimactComponent
{
    [State]
    private object user = null;

    [State]
    private bool loading = true;

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("div", new Dictionary<string, string>(), new VNode[]
        {
            loading
                ? new VElement("p", "Loading...")
                : new VElement("div", new VNode[]
                {
                    new VElement("h1", user.name),
                    new VElement("p", user.email)
                }),
            user != null
                ? new VElement("img", new Dictionary<string, string> { ["src"] = user.avatar })
                : new VText("")
        });
    }
}

// Example 3: List Rendering
[Component]
public partial class TodoList : MinimactComponent
{
    [State]
    private List<object> todos = new List<object>();

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("div", new Dictionary<string, string>(), new VNode[]
        {
            new VElement("h1", "My Todos"),
            new VElement("ul", todos.Select(todo => new VElement("li", new Dictionary<string, string>
            {
                ["key"] = todo.id.ToString()
            }, new VNode[]
            {
                new VElement("input", new Dictionary<string, string>
                {
                    ["type"] = "checkbox",
                    ["checked"] = todo.completed.ToString()
                }),
                new VElement("span", todo.text)
            })).ToArray())
        });
    }
}

// Example 4: Client State (Hybrid Rendering)
[Component]
public partial class SearchBox : MinimactComponent
{
    [State]
    private List<object> results = new List<object>();

    // Note: query is client-side only, not in C# state

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("div", new Dictionary<string, string>(), new VNode[]
        {
            // Client zone
            new VElement("input", new Dictionary<string, string>
            {
                ["data-minimact-client-scope"] = "true",
                ["data-state"] = "query",
                ["placeholder"] = "Search...",
                ["oninput"] = "HandleClientStateChange:query"
            }),

            new VElement("button", new Dictionary<string, string> { ["onclick"] = "search" }, "Search"),

            // Hybrid zone - smart span splitting
            new VElement("p", new VNode[]
            {
                new VText("Found "),
                new VElement("span", new Dictionary<string, string>
                {
                    ["data-minimact-server-scope"] = "true"
                }, results.Count.ToString()),
                new VText(" results for \""),
                new VElement("span", new Dictionary<string, string>
                {
                    ["data-minimact-client-scope"] = "true",
                    ["data-bind"] = "query"
                }),
                new VText("\"")
            }),

            // Server zone
            new VElement("ul", results.Select(r => new VElement("li", new Dictionary<string, string>
            {
                ["key"] = r.id.ToString()
            }, r.title)).ToArray())
        });
    }

    private void search()
    {
        // Server search logic
    }
}

// Example 5: Fragments
[Component]
public partial class MultiColumn : MinimactComponent
{
    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new Fragment(
            new VElement("div", new Dictionary<string, string> { ["className"] = "column" }, "Column 1"),
            new VElement("div", new Dictionary<string, string> { ["className"] = "column" }, "Column 2"),
            new VElement("div", new Dictionary<string, string> { ["className"] = "column" }, "Column 3")
        );
    }
}

// Example 6: Markdown (for blog)
[Component]
public partial class BlogPost : BlogLayoutBase
{
    [State]
    private object post = null;

    [State]
    private string content = "";

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("article", new Dictionary<string, string>(), new VNode[]
        {
            post != null
                ? new Fragment(
                    new VElement("h1", post.title),
                    new VElement("div", new Dictionary<string, string> { ["className"] = "markdown" },
                        new DivRawHtml(content))
                )
                : new VText("")
        });
    }
}

// Example 7: Complex nested structure
[Component]
public partial class Dashboard : MinimactComponent
{
    [State]
    private object stats = new { views = 0, clicks = 0, users = 0 };

    [State]
    private string selectedPeriod = "week";

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("div", new Dictionary<string, string> { ["className"] = "dashboard" }, new VNode[]
        {
            new VElement("header", new VNode[]
            {
                new VElement("h1", "Analytics Dashboard"),
                new VElement("select", new Dictionary<string, string>
                {
                    ["value"] = selectedPeriod,
                    ["onchange"] = "HandlePeriodChange"
                }, new VNode[]
                {
                    new VElement("option", new Dictionary<string, string> { ["value"] = "day" }, "Today"),
                    new VElement("option", new Dictionary<string, string> { ["value"] = "week" }, "This Week"),
                    new VElement("option", new Dictionary<string, string> { ["value"] = "month" }, "This Month")
                })
            }),

            new VElement("div", new Dictionary<string, string> { ["className"] = "stats-grid" }, new VNode[]
            {
                new VElement("div", new Dictionary<string, string> { ["className"] = "stat-card" }, new VNode[]
                {
                    new VElement("h3", "Views"),
                    new VElement("p", new Dictionary<string, string> { ["className"] = "stat-value" }, stats.views.ToString())
                }),
                new VElement("div", new Dictionary<string, string> { ["className"] = "stat-card" }, new VNode[]
                {
                    new VElement("h3", "Clicks"),
                    new VElement("p", new Dictionary<string, string> { ["className"] = "stat-value" }, stats.clicks.ToString())
                }),
                new VElement("div", new Dictionary<string, string> { ["className"] = "stat-card" }, new VNode[]
                {
                    new VElement("h3", "Users"),
                    new VElement("p", new Dictionary<string, string> { ["className"] = "stat-value" }, stats.users.ToString())
                })
            })
        });
    }

    private void HandlePeriodChange()
    {
        // Handle period selection
    }
}

// Example 8: With props (component composition)
public class CardProps
{
    public string Title { get; set; }
    public int Count { get; set; }
    public string? Icon { get; set; }
}

[Component]
public partial class Card : MinimactComponent
{
    private CardProps Props { get; set; }

    public Card(CardProps props)
    {
        Props = props;
    }

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("div", new Dictionary<string, string> { ["className"] = "card" }, new VNode[]
        {
            !string.IsNullOrEmpty(Props.Icon)
                ? new VElement("img", new Dictionary<string, string>
                {
                    ["src"] = Props.Icon,
                    ["alt"] = Props.Title
                })
                : new VText(""),
            new VElement("h3", Props.Title),
            new VElement("p", new Dictionary<string, string> { ["className"] = "count" }, Props.Count.ToString())
        });
    }
}
