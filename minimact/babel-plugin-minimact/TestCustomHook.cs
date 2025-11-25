using Minimact.AspNetCore.Core;
using Minimact.AspNetCore.Extensions;
using MinimactHelpers = Minimact.AspNetCore.Core.Minimact;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace Minimact.Components;

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
                new VText($"{(counterUI1)}", "1.2.4")
            }),
            new VElement("div", "1.3", new Dictionary<string, string> { ["class"] = "section" }, new VNode[]
            {
                new VElement("h2", "1.3.1", new Dictionary<string, string>(), "Counter 2 (starts at 10)"),
                new VElement("p", "1.3.2", new Dictionary<string, string>(), $"External count:{(count2)}"),
                new VElement("button", "1.3.3", new Dictionary<string, string> { ["onclick"] = "increment2" }, "External +1"),
                new VText($"{(counterUI2)}", "1.3.4")
            })
        });
    }

    // Helper function: useCounter
    private static dynamic useCounter(dynamic namespace, dynamic undefined)
    {
        var undefined = useState(start);
        var increment = () => setCount(count + 1);
        var decrement = () => setCount(count - 1);
        var reset = () => setCount(start);
        var ui = new VNull("");
        return new List<object> { count, increment, decrement, reset, ui };
    }
}
