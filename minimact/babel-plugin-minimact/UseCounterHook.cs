using Minimact.AspNetCore.Core;
using Minimact.AspNetCore.Extensions;
using MinimactHelpers = Minimact.AspNetCore.Core.Minimact;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace Minimact.Components;

// ============================================================
// HOOK CLASS - Generated from useCounter
// ============================================================
[Hook]
public partial class UseCounterHook : MinimactComponent
{
    // Configuration (from hook arguments)
    private dynamic start => GetState<dynamic>("_config.start");

    // Hook state
    [State]
    private dynamic count = start;

    // State setters
    private void setCount(dynamic value)
    {
        SetState(nameof(count), value);
    }

    // Hook methods
    private void increment()
    {
        return setCount((count + 1));
    }

    private void decrement()
    {
        return setCount((count - 1));
    }

    private void reset()
    {
        return setCount(start);
    }

    // Hook UI rendering
    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("div", "1", new Dictionary<string, string> { ["class"] = "counter-widget" }, new VNode[]
        {
            new VElement("button", "1.1", new Dictionary<string, string> { ["onclick"] = "decrement" }, "-"),
            new VElement("span", "1.2", new Dictionary<string, string> { ["class"] = "count-display" }, new VNode[]
            {
                new VText($"{(count)}", "1.2.1")
            }),
            new VElement("button", "1.3", new Dictionary<string, string> { ["onclick"] = "increment" }, "+"),
            new VElement("button", "1.4", new Dictionary<string, string> { ["onclick"] = "reset" }, "Reset")
        });
    }

}


[Component]
public partial class TestCustomHook : MinimactComponent
{
    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("div", "1", new Dictionary<string, string> { ["class"] = "app" }, new VNode[]
        {
            new VElement("h1", "1.1", new Dictionary<string, string>(), "Custom Hooks Test"),
            new VElement("div", "1.2", new Dictionary<string, string> { ["class"] = "section" }, new VNode[]
            {
                new VElement("h2", "1.2.1", new Dictionary<string, string>(), "Counter 1 (starts at 0)"),
                new VElement("p", "1.2.2", new Dictionary<string, string>(), $"External count:{(count1)}"),
                new VElement("button", "1.2.3", new Dictionary<string, string> { ["onclick"] = "increment1" }, "External +1"),
                new VComponentWrapper
      {
        ComponentName = "counter1",
        ComponentType = "UseCounterHook",
        HexPath = "1.2.4",
        InitialState = new Dictionary<string, object> { ["_config.param0"] = 0 }
      }
            }),
            new VElement("div", "1.3", new Dictionary<string, string> { ["class"] = "section" }, new VNode[]
            {
                new VElement("h2", "1.3.1", new Dictionary<string, string>(), "Counter 2 (starts at 10)"),
                new VElement("p", "1.3.2", new Dictionary<string, string>(), $"External count:{(count2)}"),
                new VElement("button", "1.3.3", new Dictionary<string, string> { ["onclick"] = "increment2" }, "External +1"),
                new VComponentWrapper
      {
        ComponentName = "counter2",
        ComponentType = "UseCounterHook",
        HexPath = "1.3.4",
        InitialState = new Dictionary<string, object> { ["_config.param0"] = 10 }
      }
            })
        });
    }
}
