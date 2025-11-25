using Minimact.AspNetCore.Core;
using Minimact.AspNetCore.Extensions;
using MinimactHelpers = Minimact.AspNetCore.Core.Minimact;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace Minimact.Components;

[Component]
public partial class TestImportedHook : MinimactComponent
{
    [State]
    private int count = 0;

    [State]
    private bool isOn1 = "toggle1";

    [State]
    private bool isOn2 = "toggle2";

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("div", "1", new Dictionary<string, string> { ["class"] = "app" }, new VNode[]
        {
            new VElement("h1", "1.1", new Dictionary<string, string>(), "Imported Hook Test"),
            new VElement("div", "1.2", new Dictionary<string, string> { ["class"] = "counter-section" }, new VNode[]
            {
                new VElement("h2", "1.2.1", new Dictionary<string, string>(), "Regular State (for comparison)"),
                new VElement("button", "1.2.2", new Dictionary<string, string> { ["onclick"] = "Handle0" }, $"Count:{(count)}")
            }),
            new VElement("div", "1.3", new Dictionary<string, string> { ["class"] = "toggle-section" }, new VNode[]
            {
                new VElement("h2", "1.3.1", new Dictionary<string, string>(), "Toggle 1 (starts OFF)"),
                new VElement("p", "1.3.2", new Dictionary<string, string>(), $"Status:{((new MObject(isOn1)) ? "Active" : "Inactive")}"),
                new VElement("button", "1.3.3", new Dictionary<string, string> { ["onclick"] = "toggle1" }, "External Toggle 1"),
                new VText($"{(toggleUI1)}", "1.3.4")
            }),
            new VElement("div", "1.4", new Dictionary<string, string> { ["class"] = "toggle-section" }, new VNode[]
            {
                new VElement("h2", "1.4.1", new Dictionary<string, string>(), "Toggle 2 (starts ON)"),
                new VElement("p", "1.4.2", new Dictionary<string, string>(), $"Status:{((new MObject(isOn2)) ? "Active" : "Inactive")}"),
                new VElement("button", "1.4.3", new Dictionary<string, string> { ["onclick"] = "toggle2" }, "External Toggle 2"),
                new VText($"{(toggleUI2)}", "1.4.4")
            }),
            new VElement("div", "1.5", new Dictionary<string, string> { ["class"] = "combined-state" }, new VNode[]
            {
                new VElement("p", "1.5.1", new Dictionary<string, string>(), $"Both toggles on:{(((isOn1) != null ? (isOn2) : new VNull("")) ? "YES" : "NO")}"),
                new VElement("p", "1.5.2", new Dictionary<string, string>(), $"At least one on:{(((isOn1) ?? (isOn2)) ? "YES" : "NO")}")
            })
        });
    }

    public void Handle0()
    {
        SetState(nameof(count), count + 1);
    }

    private void toggle1()
    {
        isOn1 = !isOn1;
        SetState("isOn1", isOn1);
    }

    private void toggle2()
    {
        isOn2 = !isOn2;
        SetState("isOn2", isOn2);
    }
}
