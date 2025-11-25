using Minimact.AspNetCore.Core;
using Minimact.AspNetCore.Extensions;
using MinimactHelpers = Minimact.AspNetCore.Core.Minimact;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace Minimact.Components;

[Component]
public partial class Counter : MinimactComponent
{
    [State]
    private int count = 0;

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("div", "1", new Dictionary<string, string> { ["class"] = "counter" }, new VNode[]
        {
            new VElement("h3", "1.1", new Dictionary<string, string>(), "Counter"),
            new VElement("p", "1.2", new Dictionary<string, string> { ["id"] = "counter-display" }, $"Count:{(count)}"),
            new VElement("button", "1.3", new Dictionary<string, string> { ["id"] = "child-increment-btn", ["type"] = "button", ["onclick"] = "Handle0" }, "Child Increment")
        });
    }

    public void Handle0()
    {
        SetState(nameof(count), count + 1);
    }
}

[Component]
public partial class App : MinimactComponent
{
    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        var counterValue = State["Counter.count"];

        return new VElement("div", "1", new Dictionary<string, string> { ["id"] = "app-root" }, new VNode[]
        {
            new VElement("h1", "1.1", new Dictionary<string, string>(), "Lifted State - Simple Example"),
            new VElement("div", "1.2", new Dictionary<string, string> { ["class"] = "parent-display" }, new VNode[]
            {
                new VElement("p", "1.2.1", new Dictionary<string, string>(), new VNode[]
                {
                    new VText("Parent sees counter value:", "1.2.1.1"),
                    new VElement("span", "1.2.1.2", new Dictionary<string, string> { ["id"] = "parent-sees" }, new VNode[]
                    {
                        new VText($"{(counterValue)}", "1.2.1.2.1")
                    })
                })
            }),
            new VElement("div", "1.3", new Dictionary<string, string> { ["class"] = "parent-controls" }, new VNode[]
            {
                new VElement("button", "1.3.1", new Dictionary<string, string> { ["id"] = "parent-reset-btn", ["type"] = "button", ["onclick"] = "handleParentReset" }, "Parent: Reset to 0"),
                new VElement("button", "1.3.2", new Dictionary<string, string> { ["id"] = "parent-set10-btn", ["type"] = "button", ["onclick"] = "handleParentSetTo10" }, "Parent: Set to 10")
            }),
            new VComponentWrapper
{
    ComponentName = "Counter",
    ComponentType = "Counter",
    HexPath = "1.4",
    InitialState = new Dictionary<string, object> { ["count"] = 0 }
},
            new VElement("div", "1.5", new Dictionary<string, string> { ["id"] = "status", ["class"] = "status" }, new VNode[]
            {
                new VElement("p", "1.5.1", new Dictionary<string, string>(), new VNode[]
                {
                    new VText("Counter Value:", "1.5.1.1"),
                    new VElement("span", "1.5.1.2", new Dictionary<string, string> { ["id"] = "status-value" }, new VNode[]
                    {
                        new VText($"{(counterValue)}", "1.5.1.2.1")
                    })
                })
            })
        });
    }

    public void handleParentReset()
    {
        SetState("Counter.count", 0);
    }

    public void handleParentSetTo10()
    {
        SetState("Counter.count", 10);
    }
}
